import type { useQueryClient } from '@tanstack/react-query';
import { CircleHelp, Search } from 'lucide-react';
import { useEffect } from 'react';

import { queryKeys } from '../../api/hooks';
import type { DocumentRegionsPayload, ModelsPayload, OcrProfileRecord } from '../../api/types';
import { IconButton } from '../../components/IconButton';
import type { useWorkbenchState } from '../../stores/workbenchStore';
import { followLatestRegion, setSelectedProfile } from '../../stores/workbenchStore';
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
) {
  const latestRegion = regions?.boxes.at(-1);
  useEffect(() => {
    if (!workbench.autoFollowRegions || !regions || !latestRegion) {
      return;
    }
    if (workbench.selection.regionId === latestRegion.region_id) {
      return;
    }
    followLatestRegion(regions.file_hash, regions.boxes);
  }, [latestRegion, regions, workbench.autoFollowRegions, workbench.selection.regionId]);
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

export function CommandCenter(props: {
  searchText: string;
  onSearchTextChange: (value: string) => void;
  onStartGuide: () => void;
}) {
  return (
    <div className={styles.commandCenter}>
      <Search size={15} />
      <input
        aria-label="Search documents"
        onChange={(event) => props.onSearchTextChange(event.target.value)}
        placeholder="Search documents"
        value={props.searchText}
      />
      <IconButton icon={CircleHelp} label="Start guide" onClick={props.onStartGuide} />
    </div>
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
