import { useDebouncedValue } from '@tanstack/react-pacer';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';

import {
  useCancelModelDownload,
  useDocumentPreviewImages,
  useDocumentRegions,
  useDocuments,
  useDocumentText,
  useDownloadModel,
  useIngestRuns,
  useLogs,
  useModels,
  useOpenFolderDialog,
  useRunCommand,
  useSelectModel,
  useSettings,
  useStartIngest,
  useStatus,
  useUpdateSettings,
} from '../../api/hooks';
import type { DocumentSummary, ModelAssetRecord, OcrProfileRecord } from '../../api/types';
import { useRealtimeState } from '../../realtime/realtimeStore';
import type {
  DiagnosticsRouteSearch,
  IngestRouteSearch,
  ModelRouteSearch,
  SettingsRouteSearch,
  WorkbenchRouteSearch,
} from '../../routeSearch';
import type { ActiveView } from '../../stores/workbenchStore';
import { useWorkbenchState } from '../../stores/workbenchStore';
import { useModelRouteActions } from './useModelRouteActions';
import { useWorkbenchCommands } from './useWorkbenchCommands';
import { useWorkbenchIngestActions } from './useWorkbenchIngestActions';
import {
  autoFollowEnabledForRoute,
  useRouteSearchSync,
  useRouteSearchText,
} from './useWorkbenchRouteSync';
import { useWorkbenchSelectionActions } from './useWorkbenchSelectionActions';
import {
  profileOptions,
  selectedModel,
  useAutoFollowLatestRegion,
  usePersistedWorkbenchUiSettings,
  usePersistentProfile,
  useThemeSync,
} from './WorkbenchPageSupport';
import type { WorkbenchContentActions, WorkbenchViewData } from './workbenchContentProps';
import { buildContentProps } from './workbenchContentProps';

export interface WorkbenchPageProps {
  activeView?: ActiveView;
  diagnosticsSearch?: DiagnosticsRouteSearch;
  ingestSearch?: IngestRouteSearch;
  modelDetailId?: string;
  modelScope?: 'library' | 'downloads';
  modelSearch?: ModelRouteSearch;
  settingsSearch?: SettingsRouteSearch;
  workbenchSearch?: WorkbenchRouteSearch;
}

export function useWorkbenchPageController(props: WorkbenchPageProps) {
  const activeView = props.activeView ?? 'workbench';
  const modelScope = props.modelScope ?? 'library';
  const navigate = useNavigate();
  const workbench = useWorkbenchState();
  const realtime = useRealtimeState();
  const [searchText, setSearchText] = useState(routeSearchText(props));
  const [debouncedSearch] = useDebouncedValue(searchText, { wait: 180 });
  const data = useWorkbenchData(workbench.selection.fileHash, debouncedSearch);
  const activeRunId = data.status.data?.active_run_id ?? null;
  const activeRun = data.runs.data?.runs.find((run) => run.run_id === activeRunId);
  const model = selectedModel(data.models.data);
  const profiles = profileOptions(data.models.data?.profiles, workbench.selectedProfile);
  const selectedDocument = selectedDocumentFrom(data.documents.data?.documents, workbench);

  useThemeSync(workbench.theme);
  usePersistedWorkbenchUiSettings({
    isSettingsReady: data.settings.isSuccess,
    onSave: (workbenchUi) => data.updateSettings.mutate({ workbench_ui: workbenchUi }),
    settings: data.settings.data,
    workbench,
  });
  useRouteSearchText(
    activeView,
    props.diagnosticsSearch,
    props.workbenchSearch,
    searchText,
    setSearchText,
  );
  useAutoFollowLatestRegion(
    workbench,
    data.regions.data,
    autoFollowEnabledForRoute(activeView, workbench, props.workbenchSearch),
  );
  useRouteSearchSync({
    activeView,
    workbench,
    workbenchSearch: props.workbenchSearch,
  });

  const actions = useWorkbenchActions({
    activeRunId,
    data,
    model,
    modelScope,
    navigate,
    props,
    searchText,
    setSearchText,
    workbench,
  });

  return {
    activeView,
    commandController: actions.commandController,
    contentProps: buildContentProps({
      actions,
      activeRun,
      activeRunId,
      data: viewData(data, model, profiles, selectedDocument),
      route: {
        activeView,
        diagnosticsSearch: props.diagnosticsSearch,
        ingestSearch: props.ingestSearch,
        modelDetailId: props.modelDetailId,
        modelScope,
        modelSearch: props.modelSearch,
        settingsSearch: props.settingsSearch,
      },
      workbench,
    }),
    footerProps: {
      documentCount: data.documents.data?.documents.length ?? 0,
      realtimeState: realtime.connectionState,
      selectedRoot: workbench.selectedRoot,
      status: data.status.data,
    },
    workbench,
  };
}

function useWorkbenchData(fileHash: string | undefined, debouncedSearch: string) {
  return {
    cancelModelDownload: useCancelModelDownload(),
    documents: useDocuments(debouncedSearch),
    downloadModel: useDownloadModel(),
    folderDialog: useOpenFolderDialog(),
    logs: useLogs(220),
    models: useModels(),
    previewImages: useDocumentPreviewImages(fileHash),
    regions: useDocumentRegions(fileHash),
    runs: useIngestRuns(),
    selectModel: useSelectModel(),
    settings: useSettings(),
    startIngest: useStartIngest(),
    status: useStatus(),
    stopRun: useRunCommand('stop'),
    text: useDocumentText(fileHash),
    updateSettings: useUpdateSettings(),
  };
}

interface WorkbenchActionArgs {
  activeRunId: string | null;
  data: ReturnType<typeof useWorkbenchData>;
  model?: ModelAssetRecord;
  modelScope: 'library' | 'downloads';
  navigate: ReturnType<typeof useNavigate>;
  props: WorkbenchPageProps;
  searchText: string;
  setSearchText: (value: string) => void;
  workbench: ReturnType<typeof useWorkbenchState>;
}

function useWorkbenchActions(args: WorkbenchActionArgs): WorkbenchContentActions {
  const changeProfile = usePersistentProfile(
    args.data.settings.data?.default_profile,
    args.workbench.selectedProfile,
    (profileId) => args.data.updateSettings.mutate({ default_profile: profileId }),
  );
  const ingestActions = useWorkbenchIngestActions({
    folderDialog: args.data.folderDialog,
    model: args.model,
    rootPath: args.workbench.selectedRoot,
    selectedProfile: args.workbench.selectedProfile,
    startIngest: args.data.startIngest,
  });
  const modelRouteActions = useModelRouteActions(
    args.navigate,
    args.modelScope,
    args.props.modelSearch,
  );
  const selectionActions = useWorkbenchSelectionActions({
    navigate: args.navigate,
    searchText: args.searchText,
    workbench: args.workbench,
  });
  const commandController = useWorkbenchCommands({
    cancelModelDownload: args.data.cancelModelDownload,
    downloadModel: args.data.downloadModel,
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
    changeProfile,
    commandController,
    downloadModel: args.data.downloadModel,
    ingestBusy: args.data.startIngest.isPending || args.data.folderDialog.isPending,
    modelBusy:
      args.data.downloadModel.isPending ||
      args.data.cancelModelDownload.isPending ||
      args.data.selectModel.isPending,
    selectModel: args.data.selectModel,
    settingsBusy: args.data.selectModel.isPending || args.data.updateSettings.isPending,
    stopRun: () => args.activeRunId && args.data.stopRun.mutate(args.activeRunId),
    updateRuntime: (runtimeId: string) =>
      args.data.updateSettings.mutate({ selected_runtime_id: runtimeId }),
  };
}

function viewData(
  data: ReturnType<typeof useWorkbenchData>,
  model: ModelAssetRecord | undefined,
  profiles: OcrProfileRecord[],
  selectedDocument: DocumentSummary | undefined,
): WorkbenchViewData {
  return {
    documents: data.documents.data?.documents ?? [],
    logs: data.logs.data?.logs ?? [],
    model,
    models: data.models.data,
    previewPages: data.previewImages.data?.pages ?? [],
    profiles,
    regions: data.regions.data?.boxes ?? [],
    runs: data.runs.data?.runs ?? [],
    selectedDocument,
    settings: data.settings.data,
    status: data.status.data,
    textPages: data.text.data?.pages ?? [],
  };
}

function selectedDocumentFrom(
  documents: DocumentSummary[] | undefined,
  workbench: ReturnType<typeof useWorkbenchState>,
) {
  return documents?.find((document) => document.file_hash === workbench.selection.fileHash);
}

function routeSearchText(props: WorkbenchPageProps) {
  return props.workbenchSearch?.q ?? props.diagnosticsSearch?.q ?? '';
}
