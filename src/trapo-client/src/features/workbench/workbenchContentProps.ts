import type {
  DiagnosticPipelineTaskRecord,
  DocumentSummary,
  IngestRunRecord,
  LogRecord,
  ModelAssetRecord,
  ModelsPayload,
  OcrProfileRecord,
  OverlayBox,
  PageTextRecord,
  SettingsPayload,
  StatusPayload,
} from '../../api/types';
import type {
  DiagnosticsRouteSearch,
  IngestRouteSearch,
  ModelRouteSearch,
  SearchRouteSearch,
  SettingsRouteSearch,
} from '../../routeSearch';
import type { ActiveView, useWorkbenchState } from '../../stores/workbenchStore';
import { clearFolderDialogError, setSelectedRoot, setTheme } from '../../stores/workbenchStore';
import type { useWorkbenchCommands } from './useWorkbenchCommands';
import type { WorkbenchViewContentProps } from './WorkbenchViewContent';
import type { WorkbenchExplorerFilter } from './workbenchExplorerFilter';

interface ModelDownloadInput {
  force?: boolean;
  modelId: string;
}

interface MutationLike<TInput> {
  mutate: (input: TInput) => void;
}

export interface WorkbenchContentActions {
  cancelModelDownload: MutationLike<string>;
  changeAutoFollow: (enabled: boolean) => void;
  changeModelScope: (scope: 'library' | 'downloads') => void;
  changeProfile: (profileId: string) => void;
  commandController: ReturnType<typeof useWorkbenchCommands>;
  downloadModel: MutationLike<ModelDownloadInput>;
  changeExplorerFilter: (filter: WorkbenchExplorerFilter) => void;
  changeDownloadConcurrency: (value: number) => void;
  ingestBusy: boolean;
  modelBusy: boolean;
  pickFolder: () => void;
  resumeRun: (runId: string) => void;
  restartRun: (run: IngestRunRecord) => void;
  selectDocument: (fileHash: string, pageNo?: number, runId?: string) => void;
  selectModel: MutationLike<string>;
  selectRegion: (pageNo: number, regionId: string) => void;
  settingsBusy: boolean;
  startScan: (options?: {
    embeddingAfterIngest?: boolean;
    embeddingDimension?: number;
    embeddingModelId?: string;
    reprocess?: boolean;
    textIndexAfterIngest?: boolean;
  }) => void;
  startTextIndex: (sourceRunId: string) => void;
  generateEmbedding: (input: { dimension?: number; modelId: string; sourceRunId: string }) => void;
  stopRun: (runId?: string) => void;
  updateModelRouteSearch: (patch: Partial<ModelRouteSearch>) => void;
  updateSearchRouteSearch: (patch: Partial<SearchRouteSearch>) => void;
  updateRuntime: (runtimeId: string) => void;
}

export interface WorkbenchViewData {
  documents: DocumentSummary[];
  logs: LogRecord[];
  model?: ModelAssetRecord;
  models?: ModelsPayload;
  previewPages: number[];
  pipelineTasks: DiagnosticPipelineTaskRecord[];
  profiles: OcrProfileRecord[];
  regions: OverlayBox[];
  runs: IngestRunRecord[];
  selectedDocument?: DocumentSummary;
  settings?: SettingsPayload;
  status?: StatusPayload;
  textPages: PageTextRecord[];
}

export interface WorkbenchContentRoute {
  activeView: ActiveView;
  diagnosticsSearch?: DiagnosticsRouteSearch;
  ingestSearch?: IngestRouteSearch;
  modelDetailId?: string;
  modelScope: 'library' | 'downloads';
  modelSearch?: ModelRouteSearch;
  searchSearch?: SearchRouteSearch;
  settingsSearch?: SettingsRouteSearch;
}

export function buildContentProps(args: {
  actions: WorkbenchContentActions;
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  data: WorkbenchViewData;
  explorerFilter: WorkbenchExplorerFilter;
  route: WorkbenchContentRoute;
  workbench: ReturnType<typeof useWorkbenchState>;
}): WorkbenchViewContentProps {
  return {
    ...args.data,
    activeRun: args.activeRun,
    activeRunId: args.activeRunId,
    activeView: args.route.activeView,
    diagnosticsSearch: args.route.diagnosticsSearch,
    explorerFilter: args.explorerFilter,
    folderDialogError: args.workbench.folderDialogError,
    ingestBusy: args.actions.ingestBusy,
    ingestSearch: args.route.ingestSearch,
    modelBusy: args.actions.modelBusy,
    modelDetailId: args.route.modelDetailId,
    modelScope: args.route.modelScope,
    modelSearch: args.route.modelSearch,
    onAutoFollowChange: args.actions.changeAutoFollow,
    onCancelModel: (modelId) => args.actions.cancelModelDownload.mutate(modelId),
    onDiagnosticsSearchChange: args.actions.commandController.updateDiagnosticsRouteSearch,
    onDownloadModel: (modelId, force) => args.actions.downloadModel.mutate({ force, modelId }),
    onDownloadConcurrencyChange: args.actions.changeDownloadConcurrency,
    onExplorerFilterChange: args.actions.changeExplorerFilter,
    onModelChange: (modelId) => args.actions.selectModel.mutate(modelId),
    onModelRouteSearchChange: args.actions.updateModelRouteSearch,
    onModelScopeChange: args.actions.changeModelScope,
    onOpenIngest: () => args.actions.commandController.navigateView('ingest'),
    onOpenModels: () => args.actions.commandController.navigateView('models'),
    onPickFolder: args.actions.pickFolder,
    onProfileChange: args.actions.changeProfile,
    onResumeRun: args.actions.resumeRun,
    onRootPathChange: (value) => {
      clearFolderDialogError();
      setSelectedRoot(value);
    },
    onRuntimeChange: args.actions.updateRuntime,
    onSelectDocument: args.actions.selectDocument,
    onSelectModel: (modelId) => args.actions.selectModel.mutate(modelId),
    onSelectRegion: args.actions.selectRegion,
    onStart: args.actions.startScan,
    onStartTextIndex: args.actions.startTextIndex,
    onGenerateEmbedding: args.actions.generateEmbedding,
    onRestartRun: args.actions.restartRun,
    onSearchRouteSearchChange: args.actions.updateSearchRouteSearch,
    onStop: args.actions.stopRun,
    onThemeChange: setTheme,
    rootPath: args.workbench.selectedRoot,
    selectedProfile: args.route.ingestSearch?.profile ?? args.workbench.selectedProfile,
    settingsBusy: args.actions.settingsBusy,
    settingsSearch: args.route.settingsSearch,
    searchSearch: args.route.searchSearch,
    theme: args.workbench.theme,
    workbench: args.workbench,
  };
}
