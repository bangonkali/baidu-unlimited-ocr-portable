import type {
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
  SettingsRouteSearch,
} from '../../routeSearch';
import type { ActiveView, useWorkbenchState } from '../../stores/workbenchStore';
import { setSelectedRoot, setTheme } from '../../stores/workbenchStore';
import type { useWorkbenchCommands } from './useWorkbenchCommands';
import type { WorkbenchViewContentProps } from './WorkbenchViewContent';

interface ModelDownloadInput {
  force?: boolean;
  modelId: string;
}

interface MutationLike<TInput> {
  mutate: (input: TInput) => void;
}

export interface WorkbenchContentActions {
  cancelModelDownload: MutationLike<string>;
  changeModelScope: (scope: 'library' | 'downloads') => void;
  changeProfile: (profileId: string) => void;
  clearFolderDialogError: () => void;
  commandController: ReturnType<typeof useWorkbenchCommands>;
  downloadModel: MutationLike<ModelDownloadInput>;
  folderDialogError?: string;
  ingestBusy: boolean;
  modelBusy: boolean;
  pickFolder: () => void;
  selectDocument: (fileHash: string, pageNo?: number) => void;
  selectModel: MutationLike<string>;
  selectRegion: (pageNo: number, regionId: string) => void;
  settingsBusy: boolean;
  startScan: (options?: { reprocess?: boolean }) => void;
  stopRun: () => void;
  updateModelRouteSearch: (patch: Partial<ModelRouteSearch>) => void;
  updateRuntime: (runtimeId: string) => void;
}

export interface WorkbenchViewData {
  documents: DocumentSummary[];
  logs: LogRecord[];
  model?: ModelAssetRecord;
  models?: ModelsPayload;
  previewPages: number[];
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
  settingsSearch?: SettingsRouteSearch;
}

export function buildContentProps(args: {
  actions: WorkbenchContentActions;
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  data: WorkbenchViewData;
  route: WorkbenchContentRoute;
  workbench: ReturnType<typeof useWorkbenchState>;
}): WorkbenchViewContentProps {
  return {
    ...args.data,
    activeRun: args.activeRun,
    activeRunId: args.activeRunId,
    activeView: args.route.activeView,
    diagnosticsSearch: args.route.diagnosticsSearch,
    folderDialogError: args.actions.folderDialogError,
    ingestBusy: args.actions.ingestBusy,
    ingestSearch: args.route.ingestSearch,
    modelBusy: args.actions.modelBusy,
    modelDetailId: args.route.modelDetailId,
    modelScope: args.route.modelScope,
    modelSearch: args.route.modelSearch,
    onCancelModel: (modelId) => args.actions.cancelModelDownload.mutate(modelId),
    onDiagnosticsSearchChange: args.actions.commandController.updateDiagnosticsRouteSearch,
    onDownloadModel: (modelId, force) => args.actions.downloadModel.mutate({ force, modelId }),
    onModelChange: (modelId) => args.actions.selectModel.mutate(modelId),
    onModelRouteSearchChange: args.actions.updateModelRouteSearch,
    onModelScopeChange: args.actions.changeModelScope,
    onOpenIngest: () => args.actions.commandController.navigateView('ingest'),
    onOpenModels: () => args.actions.commandController.navigateView('models'),
    onPickFolder: args.actions.pickFolder,
    onProfileChange: args.actions.changeProfile,
    onRootPathChange: (value) => {
      args.actions.clearFolderDialogError();
      setSelectedRoot(value);
    },
    onRuntimeChange: args.actions.updateRuntime,
    onSelectDocument: args.actions.selectDocument,
    onSelectModel: (modelId) => args.actions.selectModel.mutate(modelId),
    onSelectRegion: args.actions.selectRegion,
    onStart: args.actions.startScan,
    onStop: args.actions.stopRun,
    onThemeChange: setTheme,
    rootPath: args.workbench.selectedRoot,
    selectedProfile: args.workbench.selectedProfile,
    settingsBusy: args.actions.settingsBusy,
    settingsSearch: args.route.settingsSearch,
    theme: args.workbench.theme,
    workbench: args.workbench,
  };
}
