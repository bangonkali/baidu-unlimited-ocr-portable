import type { useQueryClient } from '@tanstack/react-query';
import { useEffect, useMemo, useRef } from 'react';
import { annotationIdOf } from '../../api/annotationIdentity';
import { queryKeys, useShutdownServer } from '../../api/hooks';
import type {
  DocumentRegionsPayload,
  ModelsPayload,
  OcrProfileRecord,
  SettingsPayload,
  WorkbenchUiSettingsPatch,
} from '../../api/types';
import type { ThemeMode, useWorkbenchState } from '../../stores/workbenchStore';
import {
  applyThemePreference,
  followLatestRegion,
  hydrateWorkbenchUiSettings,
  setSelectedProfile,
} from '../../stores/workbenchStore';
import { useDownloadsPane } from './downloadsPaneContext';
import { StatusBar } from './StatusBar';

export function selectedModel(models?: ModelsPayload, preferredModelId?: string) {
  return (
    models?.models.find((item) => item.model_id === preferredModelId) ??
    models?.models.find((item) => item.selected) ??
    models?.models.find((item) => item.model_id === models.selected_model_id) ??
    models?.models[0]
  );
}

export function selectedOcrModel(models?: ModelsPayload, preferredModelId?: string) {
  const ocrModels = models?.models.filter((item) => item.model_kind !== 'embedding') ?? [];
  return (
    ocrModels.find((item) => item.model_id === preferredModelId) ??
    ocrModels.find((item) => item.selected) ??
    ocrModels.find((item) => item.model_id === models?.selected_model_id) ??
    ocrModels[0]
  );
}

export function profileOptions(profiles: OcrProfileRecord[] | undefined, selectedProfile: string) {
  return profiles?.length
    ? profiles
    : [
        {
          key: selectedProfile,
          label: 'Experimental exact-prefill Q4',
          engine_name: 'Unlimited-OCR FFI',
          description: '',
          default_max_tokens: 8192,
        },
      ];
}

export function useAutoFollowLatestRegion(
  workbench: ReturnType<typeof useWorkbenchState>,
  regions?: DocumentRegionsPayload,
  enabled = workbench.autoFollowRegions,
) {
  useEffect(() => {
    if (!enabled || !regions || !shouldFollowLatestRegion(workbench.selection, regions)) {
      return;
    }
    followLatestRegion(regions.file_hash, regions.boxes, regions.run_id);
  }, [enabled, regions, workbench.selection]);
}

export function shouldFollowLatestRegion(
  selection: { fileHash?: string; pageNo: number; regionId?: string; runId?: string },
  regions?: DocumentRegionsPayload,
) {
  const latestRegion = regions?.boxes.at(-1);
  if (!regions || !latestRegion) {
    return false;
  }
  if (selection.regionId === annotationIdOf(latestRegion)) {
    return false;
  }
  if (selection.runId && regions.run_id && selection.runId !== regions.run_id) {
    return false;
  }
  if (selection.fileHash === regions.file_hash && latestRegion.page_no < selection.pageNo) {
    return false;
  }
  return true;
}

export function WorkbenchFooter(props: {
  accelerator?: string;
  documentCount: number;
  logPath?: string;
  realtimeState: string;
  runState: string;
  runtimePlatform?: string;
  selectedRoot: string;
}) {
  const downloadsPane = useDownloadsPane();
  const shutdown = useShutdownServer();
  return (
    <StatusBar
      downloadsActiveCount={downloadsPane.activeFileCount}
      downloadsOpen={downloadsPane.isOpen}
      documentCount={props.documentCount}
      host={window.location.host}
      logPath={props.logPath}
      onDownloadsToggle={downloadsPane.toggle}
      onShutdown={() => shutdown.mutate()}
      realtimeState={props.realtimeState}
      runState={props.runState}
      runtime={`${props.runtimePlatform ?? 'windows-x86_64-cuda13'} / ${
        props.accelerator ?? 'cuda'
      }`}
      selectedRoot={props.selectedRoot}
      shutdownPending={shutdown.isPending}
    />
  );
}

export function useThemeSync(theme: ThemeMode) {
  useEffect(() => {
    applyThemePreference(theme);
  }, [theme]);
}

export function usePersistedWorkbenchUiSettings(args: {
  isSettingsReady: boolean;
  onSave: (workbenchUi: WorkbenchUiSettingsPatch) => void;
  settings?: SettingsPayload;
  workbench: ReturnType<typeof useWorkbenchState>;
}) {
  const hydratedJsonRef = useRef('');
  const persistedJsonRef = useRef('');
  const skipNextPersistRef = useRef(false);
  const { autoFollowRegions, labelsVisible, overlayVisible, panesCollapsed, theme } =
    args.workbench;
  const uiPatch = useMemo(
    () => ({
      auto_follow_regions: autoFollowRegions,
      labels_visible: labelsVisible,
      overlay_visible: overlayVisible,
      panes_collapsed: panesCollapsed,
      theme,
    }),
    [autoFollowRegions, labelsVisible, overlayVisible, panesCollapsed, theme],
  );
  const uiPatchJson = useMemo(() => serializeWorkbenchUiSettings(uiPatch), [uiPatch]);

  useEffect(() => {
    const workbenchUi = args.settings?.workbench_ui;
    if (!workbenchUi) {
      return;
    }
    const serialized = serializeWorkbenchUiSettings(workbenchUi);
    if (serialized === hydratedJsonRef.current) {
      return;
    }
    hydratedJsonRef.current = serialized;
    persistedJsonRef.current = serialized;
    skipNextPersistRef.current = true;
    hydrateWorkbenchUiSettings(workbenchUi);
  }, [args.settings?.workbench_ui]);

  useEffect(() => {
    if (!args.isSettingsReady) {
      return;
    }
    if (skipNextPersistRef.current) {
      skipNextPersistRef.current = false;
      return;
    }
    if (uiPatchJson === persistedJsonRef.current) {
      return;
    }
    persistedJsonRef.current = uiPatchJson;
    args.onSave(uiPatch);
  }, [args.isSettingsReady, args.onSave, uiPatch, uiPatchJson]);
}

function serializeWorkbenchUiSettings(value: WorkbenchUiSettingsPatch) {
  return JSON.stringify({
    auto_follow_regions: value.auto_follow_regions,
    labels_visible: value.labels_visible,
    overlay_visible: value.overlay_visible,
    panes_collapsed: {
      details: value.panes_collapsed?.details,
      diagnostics: value.panes_collapsed?.diagnostics,
      explorer: value.panes_collapsed?.explorer,
    },
    theme: value.theme,
  });
}

export function usePersistentProfile(
  defaultProfile: string | undefined,
  selectedProfile: string,
  saveProfile: (profileId: string) => void,
) {
  useEffect(() => {
    if (defaultProfile && defaultProfile !== selectedProfile) {
      setSelectedProfile(defaultProfile);
    }
  }, [defaultProfile, selectedProfile]);

  return (profileId: string) => {
    setSelectedProfile(profileId);
    saveProfile(profileId);
  };
}

export function useWorkbenchRefresh(queryClient: ReturnType<typeof useQueryClient>) {
  return () => {
    void queryClient.invalidateQueries({ queryKey: queryKeys.status });
    void queryClient.invalidateQueries({ queryKey: queryKeys.models });
    void queryClient.invalidateQueries({ queryKey: queryKeys.runs });
    void queryClient.invalidateQueries({ queryKey: ['documents'] });
    void queryClient.invalidateQueries({ queryKey: queryKeys.logs });
  };
}
