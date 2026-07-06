import { useDebouncedValue } from '@tanstack/react-pacer';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useState } from 'react';

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
  useResumeRun,
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
import {
  setSelectedProfile,
  setSelectedRoot,
  useWorkbenchState,
} from '../../stores/workbenchStore';
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
  const queryClient = useQueryClient();
  const workbench = useWorkbenchState();
  const realtime = useRealtimeState();
  const [searchText, setSearchText] = useState(routeSearchText(props));
  const [debouncedSearch] = useDebouncedValue(searchText, { wait: 180 });
  const data = useWorkbenchData(
    workbench.selection.fileHash,
    props.workbenchSearch?.run,
    debouncedSearch,
  );
  const activeRunId = data.status.data?.active_run_id ?? activeRunIdFromRuns(data.runs.data?.runs);
  const activeRun = data.runs.data?.runs.find((run) => run.run_id === activeRunId);
  const explorerFilter = explorerFilterFromSearch(
    props.workbenchSearch,
    data.runs.data?.runs ?? [],
  );
  useIngestRoutePrefill(activeView, props.ingestSearch);
  const selectedProfile = props.ingestSearch?.profile ?? workbench.selectedProfile;
  const model = selectedModel(data.models.data, props.ingestSearch?.model);
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
    startOcr: () => startOcrEntry({ model, navigate, selectedProfile }),
    workbench,
  };
}

export function useWorkbenchData(
  fileHash: string | undefined,
  runId: string | undefined,
  debouncedSearch: string,
) {
  const runs = useIngestRuns();
  const documentRunId = runId ?? latestRunIdFromRuns(runs.data?.runs);
  return {
    cancelModelDownload: useCancelModelDownload(),
    documents: useDocuments(debouncedSearch),
    downloadModel: useDownloadModel(),
    folderDialog: useOpenFolderDialog(),
    logs: useLogs(220),
    models: useModels(),
    previewImages: useDocumentPreviewImages(fileHash),
    documentRunId,
    regions: useDocumentRegions(fileHash, documentRunId),
    resumeRun: useResumeRun(),
    runs,
    selectModel: useSelectModel(),
    settings: useSettings(),
    startIngest: useStartIngest(),
    status: useStatus(),
    stopRun: useRunCommand('stop'),
    text: useDocumentText(fileHash, documentRunId),
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
    textPages: visibleTextPages(data.text.data?.pages ?? [], selectedDocument),
  };
}

function selectedDocumentFrom(
  documents: DocumentSummary[] | undefined,
  workbench: ReturnType<typeof useWorkbenchState>,
) {
  return documents?.find((document) => document.file_hash === workbench.selection.fileHash); // skylos: ignore[SKY-D253] file_hash is a public document identifier, not a secret token.
}
