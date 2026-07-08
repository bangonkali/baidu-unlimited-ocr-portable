import { useDebouncedValue } from '@tanstack/react-pacer';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useState } from 'react';

import type { DocumentSummary, ModelAssetRecord, OcrProfileRecord } from '../../api/types';
import { useRealtimeState } from '../../realtime/realtimeStore';
import type {
  DiagnosticsRouteSearch,
  IngestRouteSearch,
  ModelRouteSearch,
  RootRouteSearch,
  SearchRouteSearch,
  SettingsRouteSearch,
  WorkbenchRouteSearch,
} from '../../routeSearch';
import type { ActiveView } from '../../stores/workbenchStore';
import {
  setSelectedProfile,
  setSelectedRoot,
  setSelection,
  useWorkbenchState,
} from '../../stores/workbenchStore';
import { primaryPipelineActivity } from './pipelineTaskActivity';
import { startOcrEntry } from './startOcrEntry';
import { visibleTextPages } from './textPreviewPages';
import { useSelectedPageReplay } from './useOcrReplayHydration';
import { useWorkbenchData } from './useWorkbenchData';
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
import { explorerFilterFromSearch } from './workbenchExplorerFilter';
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

function useDefaultEngineRouteSync(args: {
  activeView: ActiveView;
  data: ReturnType<typeof useWorkbenchData>;
  navigate: ReturnType<typeof useNavigate>;
  props: WorkbenchPageProps;
  searchText: string;
  workbench: ReturnType<typeof useWorkbenchState>;
}) {
  const { activeView, data, navigate, props, searchText, workbench } = args;
  const selectedRunEngineId = data.selectedRunEngineId;
  const documentRunId = data.documentRunId;
  const fileHash = workbench.selection.fileHash ?? data.selectedDocument?.file_hash;
  const pageNo = workbench.selection.fileHash
    ? workbench.selection.pageNo
    : (data.selectedDocument?.current_page ?? 1);

  useEffect(() => {
    if (activeView !== 'workbench' || props.workbenchSearch?.result || !selectedRunEngineId) {
      return;
    }
    setSelection({
      fileHash,
      pageNo,
      regionId: workbench.selection.regionId,
      runEngineId: selectedRunEngineId,
      runId: documentRunId,
    });
    void navigate({
      replace: true,
      search: (current) => ({
        ...(current as RootRouteSearch & Record<string, unknown>),
        file: fileHash,
        follow: workbench.autoFollowRegions,
        page: fileHash ? pageNo : undefined,
        q: searchText.trim() || undefined,
        region: workbench.selection.regionId,
        result: selectedRunEngineId,
        run: documentRunId,
      }),
      to: '/workbench',
    });
  }, [
    activeView,
    documentRunId,
    fileHash,
    navigate,
    pageNo,
    props.workbenchSearch?.result,
    searchText,
    selectedRunEngineId,
    workbench.autoFollowRegions,
    workbench.selection.regionId,
  ]);
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
  const data = useWorkbenchData({
    debouncedSearch,
    fileHash: workbench.selection.fileHash,
    resultId: props.workbenchSearch?.result,
    runId: props.workbenchSearch?.run ?? props.searchSearch?.run ?? workbench.selection.runId,
    selectionRunEngineId: workbench.selection.runEngineId,
    selectionRunId: workbench.selection.runId,
  });
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
    runEngineId: data.selectedRunEngineId,
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
  useDefaultEngineRouteSync({
    activeView,
    data,
    navigate,
    props,
    searchText,
    workbench,
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
    diagnosticWorkUnits: data.progress.data?.work_units ?? [],
    documents: data.documents.data?.documents ?? [],
    enginePresets: data.ingestEngines.data?.engines ?? [],
    logs: data.logs.data?.logs ?? [],
    model,
    models: data.models.data,
    previewPages: data.previewImages.data?.pages ?? [],
    previewResults: data.previewResultOptions,
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
