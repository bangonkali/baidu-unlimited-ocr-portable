import type { useNavigate } from '@tanstack/react-router';

import type { ModelAssetRecord } from '../../api/types';
import type { useWorkbenchState } from '../../stores/workbenchStore';
import { setAutoFollowRegions, setSelection } from '../../stores/workbenchStore';
import { useDownloadModelWithPane } from './downloadsPaneContext';
import { useModelRouteActions } from './useModelRouteActions';
import { useWorkbenchCommands } from './useWorkbenchCommands';
import { useWorkbenchIngestActions } from './useWorkbenchIngestActions';
import type { useWorkbenchData, WorkbenchPageProps } from './useWorkbenchPageController';
import {
  routeSearchFromSelection,
  useWorkbenchSelectionActions,
} from './useWorkbenchSelectionActions';
import { usePersistentProfile } from './WorkbenchPageSupport';
import type { WorkbenchContentActions } from './workbenchContentProps';
import type { WorkbenchExplorerFilter } from './workbenchExplorerFilter';
import { firstDocumentForRun, latestRunIdFromRuns } from './workbenchExplorerFilter';

interface WorkbenchActionArgs {
  activeRunId: string | null;
  data: ReturnType<typeof useWorkbenchData>;
  explorerFilter: WorkbenchExplorerFilter;
  model?: ModelAssetRecord;
  modelScope: 'library' | 'downloads';
  navigate: ReturnType<typeof useNavigate>;
  props: WorkbenchPageProps;
  selectedProfile: string;
  searchText: string;
  setSearchText: (value: string) => void;
  workbench: ReturnType<typeof useWorkbenchState>;
}

export function useWorkbenchActions(args: WorkbenchActionArgs): WorkbenchContentActions {
  const changeProfile = usePersistentProfile(
    args.data.settings.data?.default_profile,
    args.workbench.selectedProfile,
    (profileId) => args.data.updateSettings.mutate({ default_profile: profileId }),
  );
  const ingestActions = useWorkbenchIngestActions({
    engineId: args.props.ingestSearch?.engine,
    folderDialog: args.data.folderDialog,
    model: args.model,
    navigate: args.navigate,
    rootPath: args.workbench.selectedRoot,
    runtimeId: args.props.ingestSearch?.runtime,
    selectedProfile: args.selectedProfile,
    startIngest: args.data.startIngest,
  });
  const modelRouteActions = useModelRouteActions(
    args.navigate,
    args.modelScope,
    args.props.modelSearch,
  );
  const selectionActions = useWorkbenchSelectionActions({
    navigate: args.navigate,
    runScope: args.explorerFilter.scope === 'all' ? 'all' : undefined,
    searchText: args.searchText,
    workbench: args.workbench,
  });
  const downloadModel = useDownloadModelWithPane(args.data.downloadModel);
  const commandController = useWorkbenchCommands({
    cancelModelDownload: args.data.cancelModelDownload,
    downloadModel,
    models: args.data.models.data,
    navigate: args.navigate,
    selectModel: args.data.selectModel,
    setSearchText: args.setSearchText,
    status: args.data.status.data,
    workbench: args.workbench,
  });

  return {
    ...ingestActions,
    ...modelRouteActions,
    ...selectionActions,
    cancelModelDownload: args.data.cancelModelDownload,
    changeDownloadConcurrency: (value: number) =>
      args.data.updateSettings.mutate({ download_concurrency: value }),
    changeExplorerFilter: (filter) => changeExplorerFilter(args, filter),
    changeProfile,
    commandController,
    downloadModel,
    generateEmbedding: ({ dimension, modelId, sourceRunId }) =>
      args.data.generateEmbedding.mutate({
        dimension,
        model_id: modelId,
        source_run_id: sourceRunId,
      }),
    ingestBusy:
      args.data.startIngest.isPending ||
      args.data.folderDialog.isPending ||
      args.data.startTextIndex.isPending ||
      args.data.generateEmbedding.isPending,
    modelBusy:
      args.data.downloadModel.isPending ||
      args.data.cancelModelDownload.isPending ||
      args.data.selectModel.isPending,
    resumeRun: (runId: string) => args.data.resumeRun.mutate(runId),
    restartRun: (run) =>
      void args.navigate({
        search: {
          engine: run.engine_id,
          model: run.model_id,
          profile: run.profile_id,
          restart: run.run_id,
          root: run.root_path,
          runtime: run.runtime_id,
        },
        to: '/ingest/start',
      }),
    selectModel: args.data.selectModel,
    settingsBusy: args.data.selectModel.isPending || args.data.updateSettings.isPending,
    startTextIndex: (sourceRunId: string) =>
      args.data.startTextIndex.mutate({ source_run_id: sourceRunId }),
    stopRun: (runId?: string) => {
      const targetRunId = runId ?? args.activeRunId;
      if (targetRunId) {
        args.data.stopRun.mutate(targetRunId);
      }
    },
    updateRuntime: (runtimeId: string) =>
      args.data.updateSettings.mutate({ selected_runtime_id: runtimeId }),
    updateSearchRouteSearch: (patch) =>
      void args.navigate({
        search: (current) => ({ ...current, ...patch }),
        to: '/search',
      }),
  };
}

function changeExplorerFilter(args: WorkbenchActionArgs, filter: WorkbenchExplorerFilter) {
  const runs = args.data.runs.data?.runs ?? [];
  const documents = args.data.documents.data?.documents ?? [];
  const runId =
    filter.scope === 'run'
      ? filter.runId
      : (args.explorerFilter.runId ?? latestRunIdFromRuns(runs));
  const firstDocument = firstDocumentForRun(documents, runs, runId);
  const nextSelection = {
    fileHash: firstDocument?.file_hash,
    pageNo: firstDocument?.current_page ?? 1,
    regionId: undefined,
    runId,
  };
  setAutoFollowRegions(false);
  setSelection(nextSelection);
  void args.navigate({
    replace: true,
    search: () =>
      routeSearchFromSelection(
        { ...args.workbench, autoFollowRegions: false },
        nextSelection,
        args.searchText,
        { runScope: filter.scope === 'all' ? 'all' : undefined },
      ),
    to: '/workbench',
  });
}
