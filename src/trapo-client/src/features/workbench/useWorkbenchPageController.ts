import { useDebouncedValue } from '@tanstack/react-pacer';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useState } from 'react';

import {
  useCancelModelDownload,
  useDiagnosticProgress,
  useDocumentPreviewImages,
  useDocumentRegions,
  useDocuments,
  useDocumentText,
  useDownloadModel,
  useGenerateEmbedding,
  useIngestEngines,
  useIngestPreviewResults,
  useIngestRuns,
  useLogs,
  useModels,
  useOpenFolderDialog,
  useResumeRun,
  useRunCommand,
  useSelectModel,
  useSettings,
  useStartIngest,
  useStartTextIndex,
  useStatus,
  useUpdateSettings,
  useUsedEmbeddingModels,
} from '../../api/hooks';
import type { DocumentSummary, ModelAssetRecord, OcrProfileRecord } from '../../api/types';
import { useRealtimeState } from '../../realtime/realtimeStore';
import type {
  DiagnosticsRouteSearch,
  IngestRouteSearch,
  ModelRouteSearch,
  SearchRouteSearch,
  SettingsRouteSearch,
  WorkbenchRouteSearch,
} from '../../routeSearch';
import type { ActiveView } from '../../stores/workbenchStore';
import {
  setSelectedProfile,
  setSelectedRoot,
  useWorkbenchState,
} from '../../stores/workbenchStore';
import { primaryPipelineActivity } from './pipelineTaskActivity';
import { startOcrEntry } from './startOcrEntry';
import { visibleTextPages } from './textPreviewPages';
import { useSelectedPageReplay } from './useOcrReplayHydration';
import { useWorkbenchActions } from './useWorkbenchPageActions';
import {
  autoFollowEnabledForRoute,
  useRouteSearchSync,
  useRouteSearchText,
} from './useWorkbenchRouteSync';
import {
  profileOptions,
  selectedModel,
  selectedOcrModel,
  useAutoFollowLatestRegion,
  usePersistedWorkbenchUiSettings,
  useThemeSync,
} from './WorkbenchPageSupport';
import type { WorkbenchViewData } from './workbenchContentProps';
import { buildContentProps } from './workbenchContentProps';
import { explorerFilterFromSearch, latestRunIdFromRuns } from './workbenchExplorerFilter';
import { activeRunIdFromRuns, isActiveDocumentStatus, routeSearchText } from './workbenchRunState';

export interface WorkbenchPageProps {
  activeView?: ActiveView;
  diagnosticsSearch?: DiagnosticsRouteSearch;
  diagnosticWorkUnitId?: string;
  ingestSearch?: IngestRouteSearch;
  modelDetailId?: string;
  modelScope?: 'library' | 'downloads';
  modelSearch?: ModelRouteSearch;
  searchSearch?: SearchRouteSearch;
  settingsSearch?: SettingsRouteSearch;
  workbenchSearch?: WorkbenchRouteSearch;
}

export function useWorkbenchPageController(props: WorkbenchPageProps) {
  const activeView = props.activeView ?? 'workbench';
  const modelScope = props.modelScope ?? 'library';
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const workbench = useWorkbenchState();
  const realtime = useRealtimeState();
  const [searchText, setSearchText] = useState(routeSearchText(props));
  const [debouncedSearch] = useDebouncedValue(searchText, { wait: 180 });
  const data = useWorkbenchData(
    workbench.selection.fileHash,
    props.workbenchSearch?.run ?? props.searchSearch?.run,
    props.workbenchSearch?.result,
    debouncedSearch,
  );
  const activeRunId = data.status.data?.active_run_id ?? activeRunIdFromRuns(data.runs.data?.runs);
  const activeRun = data.runs.data?.runs.find((run) => run.run_id === activeRunId);
  const pipelineTasks = data.progress.data?.pipeline_tasks ?? [];
  const pipelineTask = primaryPipelineActivity(pipelineTasks);
  const explorerFilter = explorerFilterFromSearch(
    props.workbenchSearch,
    data.runs.data?.runs ?? [],
  );
  useIngestRoutePrefill(activeView, props.ingestSearch);
  const selectedProfile = props.ingestSearch?.profile ?? workbench.selectedProfile;
  const model =
    activeView === 'ingest'
      ? selectedOcrModel(data.models.data, props.ingestSearch?.model)
      : selectedModel(data.models.data, props.ingestSearch?.model);
  const profiles = profileOptions(data.models.data?.profiles, selectedProfile);
  const selectedDocument = selectedDocumentFrom(data.documents.data?.documents, workbench);
  useSelectedPageReplay({
    enabled:
      workbench.selectionSource === 'manual' &&
      (selectedDocument ? isActiveDocumentStatus(selectedDocument.status) : false),
    fileHash: workbench.selection.fileHash,
    pageNo: workbench.selection.pageNo,
    queryClient,
    runId: data.documentRunId,
  });

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
    selectedProfile,
    searchText,
    setSearchText,
    workbench,
    explorerFilter,
  });

  return {
    activeView,
    commandController: actions.commandController,
    contentProps: buildContentProps({
      actions,
      activeRun,
      activeRunId,
      data: viewData(data, model, profiles, selectedDocument),
      explorerFilter,
      route: {
        activeView,
        diagnosticsSearch: props.diagnosticsSearch,
        diagnosticWorkUnitId: props.diagnosticWorkUnitId,
        ingestSearch: props.ingestSearch,
        modelDetailId: props.modelDetailId,
        modelScope,
        modelSearch: props.modelSearch,
        searchSearch: props.searchSearch,
        settingsSearch: props.settingsSearch,
      },
      workbench,
    }),
    footerProps: {
      documentCount: data.documents.data?.documents.length ?? 0,
      pipelineTask,
      realtimeState: realtime.connectionState,
      selectedRoot: workbench.selectedRoot,
      status: data.status.data,
    },
    startOcr: () => startOcrEntry({ model, navigate, selectedProfile }),
    workbench,
  };
}

export function useWorkbenchData(
  fileHash: string | undefined,
  runId: string | undefined,
  resultId: string | undefined,
  debouncedSearch: string,
) {
  const runs = useIngestRuns();
  const documentRunId = runId ?? latestRunIdFromRuns(runs.data?.runs);
  const previewResults = useIngestPreviewResults(documentRunId, fileHash);
  const runPreviewResults =
    runs.data?.runs.find((run) => run.run_id === documentRunId)?.preview_results ?? [];
  const selectedRunEngineId =
    resultId ??
    previewResults.data?.results[0]?.run_engine_id ??
    runPreviewResults[0]?.run_engine_id;
  return {
    cancelModelDownload: useCancelModelDownload(),
    documents: useDocuments(debouncedSearch),
    downloadModel: useDownloadModel(),
    folderDialog: useOpenFolderDialog(),
    generateEmbedding: useGenerateEmbedding(),
    ingestEngines: useIngestEngines(),
    logs: useLogs(220),
    models: useModels(),
    progress: useDiagnosticProgress(undefined, 5000, 1500),
    previewImages: useDocumentPreviewImages(fileHash),
    previewResults,
    runPreviewResults,
    documentRunId,
    selectedRunEngineId,
    regions: useDocumentRegions(fileHash, documentRunId, selectedRunEngineId),
    resumeRun: useResumeRun(),
    runs,
    selectModel: useSelectModel(),
    settings: useSettings(),
    startIngest: useStartIngest(),
    startTextIndex: useStartTextIndex(),
    status: useStatus(),
    stopRun: useRunCommand('stop'),
    text: useDocumentText(fileHash, documentRunId, selectedRunEngineId),
    usedEmbeddingModels: useUsedEmbeddingModels(),
    updateSettings: useUpdateSettings(),
  };
}

function useIngestRoutePrefill(activeView: ActiveView, search?: IngestRouteSearch) {
  useEffect(() => {
    if (activeView !== 'ingest') {
      return;
    }
    if (search?.root) {
      setSelectedRoot(search.root);
    }
    if (search?.profile) {
      setSelectedProfile(search.profile);
    }
  }, [activeView, search?.profile, search?.root]);
}

function viewData(
  data: ReturnType<typeof useWorkbenchData>,
  model: ModelAssetRecord | undefined,
  profiles: OcrProfileRecord[],
  selectedDocument: DocumentSummary | undefined,
): WorkbenchViewData {
  return {
    documents: data.documents.data?.documents ?? [],
    enginePresets: data.ingestEngines.data?.engines ?? [],
    logs: data.logs.data?.logs ?? [],
    model,
    models: data.models.data,
    previewPages: data.previewImages.data?.pages ?? [],
    previewResults: data.previewResults.data?.results.length
      ? data.previewResults.data.results
      : data.runPreviewResults,
    profiles,
    pipelineTasks: data.progress.data?.pipeline_tasks ?? [],
    regions: data.regions.data?.boxes ?? [],
    runs: data.runs.data?.runs ?? [],
    selectedDocument,
    selectedRunEngineId: data.selectedRunEngineId,
    settings: data.settings.data,
    status: data.status.data,
    textPages: visibleTextPages(data.text.data?.pages ?? [], selectedDocument),
  };
}

function selectedDocumentFrom(
  documents: DocumentSummary[] | undefined,
  workbench: ReturnType<typeof useWorkbenchState>,
) {
  return documents?.find((document) => document.file_hash === workbench.selection.fileHash); // skylos: ignore[SKY-D253] file_hash is a public document identifier, not a secret token.
}
