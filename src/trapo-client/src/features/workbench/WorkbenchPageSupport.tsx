import type { useQueryClient } from '@tanstack/react-query';
import { CircleHelp, PanelBottom, PanelLeft, PanelRight, Search } from 'lucide-react';
import { useEffect, useMemo, useRef } from 'react';

import { queryKeys } from '../../api/hooks';
import type {
  DocumentRegionsPayload,
  ModelsPayload,
  OcrProfileRecord,
  SettingsPayload,
  WorkbenchUiSettingsPatch,
} from '../../api/types';
import { IconButton } from '../../components/IconButton';
import type {
  ActiveView,
  ThemeMode,
  useWorkbenchState,
  WorkbenchPaneId,
  WorkbenchPaneState,
} from '../../stores/workbenchStore';
import {
  applyThemePreference,
  followLatestRegion,
  hydrateWorkbenchUiSettings,
  setSelectedProfile,
} from '../../stores/workbenchStore';
import { StatusBar } from './StatusBar';
import styles from './WorkbenchPage.module.css';

export function selectedModel(models?: ModelsPayload) {
  return (
    models?.models.find((item) => item.selected) ??
    models?.models.find((item) => item.model_id === models.selected_model_id) ??
    models?.models[0]
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
  const latestRegion = regions?.boxes.at(-1);
  useEffect(() => {
    if (!enabled || !regions || !latestRegion) {
      return;
    }
    if (workbench.selection.regionId === latestRegion.region_id) {
      return;
    }
    followLatestRegion(regions.file_hash, regions.boxes);
  }, [enabled, latestRegion, regions, workbench.selection.regionId]);
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
  return (
    <StatusBar
      documentCount={props.documentCount}
      host={window.location.host}
      logPath={props.logPath}
      realtimeState={props.realtimeState}
      runState={props.runState}
      runtime={`${props.runtimePlatform ?? 'windows-x86_64-cuda13'} / ${
        props.accelerator ?? 'cuda'
      }`}
      selectedRoot={props.selectedRoot}
    />
  );
}

export function WorkbenchHeader(props: {
  activeView: ActiveView;
  panesCollapsed: WorkbenchPaneState;
  theme: ThemeMode;
  onCommandOpen: () => void;
  onStartGuide: () => void;
  onTogglePane: (pane: WorkbenchPaneId) => void;
}) {
  const showPaneControls = props.activeView === 'workbench';
  return (
    <div className={styles.topHeader}>
      <div className={styles.headerSpacer} />
      <button className={styles.commandTrigger} onClick={props.onCommandOpen} type="button">
        <Search size={15} />
        <span>Search commands, models, routes, or documents</span>
        <kbd>Ctrl K</kbd>
      </button>
      <div className={styles.headerActions}>
        {showPaneControls ? (
          <>
            <PaneToggleButton
              active={!props.panesCollapsed.explorer}
              icon={PanelLeft}
              label="Toggle Explorer"
              onClick={() => props.onTogglePane('explorer')}
            />
            <PaneToggleButton
              active={!props.panesCollapsed.diagnostics}
              icon={PanelBottom}
              label="Toggle Diagnostics"
              onClick={() => props.onTogglePane('diagnostics')}
            />
            <PaneToggleButton
              active={!props.panesCollapsed.details}
              icon={PanelRight}
              label="Toggle Details"
              onClick={() => props.onTogglePane('details')}
            />
          </>
        ) : null}
        <IconButton icon={CircleHelp} label="Start guide" onClick={props.onStartGuide} />
      </div>
    </div>
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

function PaneToggleButton(props: {
  active: boolean;
  icon: typeof PanelLeft;
  label: string;
  onClick: () => void;
}) {
  const Icon = props.icon;
  return (
    <button
      aria-label={props.label}
      aria-pressed={props.active}
      className={styles.paneToggle}
      onClick={props.onClick}
      title={props.label}
      type="button"
    >
      <Icon size={16} strokeWidth={1.9} />
    </button>
  );
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
