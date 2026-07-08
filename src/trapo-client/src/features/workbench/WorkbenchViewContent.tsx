import type {
  DiagnosticPipelineTaskRecord,
  DiagnosticWorkUnitRecord,
  DocumentSummary,
  IngestEnginePresetRecord,
  IngestEngineSelection,
  IngestPreviewResultRecord,
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
import type { ActiveView, ThemeMode, useWorkbenchState } from '../../stores/workbenchStore';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { IngestStartPanel } from './IngestStartPanel';
import { ModelDetailPanel } from './ModelDetailPanel';
import { ModelManager } from './ModelManager';
import { SearchView } from './SearchView';
import { SettingsPanel } from './SettingsPanel';
import { WorkbenchPanels } from './WorkbenchPanels';
import type { WorkbenchExplorerFilter } from './workbenchExplorerFilter';

export interface WorkbenchViewContentProps {
  activeView: ActiveView;
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  diagnosticWorkUnitId?: string;
  diagnosticsSearch?: DiagnosticsRouteSearch;
  documents: DocumentSummary[];
  explorerFilter: WorkbenchExplorerFilter;
  enginePresets: IngestEnginePresetRecord[];
  folderDialogError?: string;
  ingestBusy: boolean;
  ingestSearch?: IngestRouteSearch;
  logs: LogRecord[];
  model?: ModelAssetRecord;
  modelBusy: boolean;
  modelDetailId?: string;
  modelScope: 'library' | 'downloads';
  modelSearch?: ModelRouteSearch;
  searchSearch?: SearchRouteSearch;
  models?: ModelsPayload;
  previewPages: number[];
  previewResults: IngestPreviewResultRecord[];
  diagnosticWorkUnits: DiagnosticWorkUnitRecord[];
  pipelineTasks: DiagnosticPipelineTaskRecord[];
  profiles: OcrProfileRecord[];
  regions: OverlayBox[];
  rootPath: string;
  runs: IngestRunRecord[];
  selectedDocument?: DocumentSummary;
  selectedRunEngineId?: string;
  selectedProfile: string;
  settings?: SettingsPayload;
  settingsBusy: boolean;
  settingsSearch?: SettingsRouteSearch;
  status?: StatusPayload;
  theme: ThemeMode;
  textPages: PageTextRecord[];
  workbench: ReturnType<typeof useWorkbenchState>;
  onAutoFollowChange: (enabled: boolean) => void;
  onCancelModel: (modelId: string) => void;
  onDiagnosticsSearchChange: (patch: Partial<DiagnosticsRouteSearch>) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onDownloadConcurrencyChange: (value: number) => void;
  onExplorerFilterChange: (filter: WorkbenchExplorerFilter) => void;
  onModelChange: (modelId: string) => void;
  onModelRouteSearchChange: (patch: Partial<ModelRouteSearch>) => void;
  onModelScopeChange: (scope: 'library' | 'downloads') => void;
  onOpenIngest: () => void;
  onOpenModels: () => void;
  onPickFolder: () => void;
  onProfileChange: (profileId: string) => void;
  onResumeRun: (runId: string) => void;
  onRestartRun: (run: IngestRunRecord) => void;
  onRootPathChange: (value: string) => void;
  onRuntimeChange: (runtimeId: string) => void;
  onSelectModel: (modelId: string) => void;
  onSelectDocument: (
    fileHash: string,
    pageNo?: number,
    runId?: string,
    runEngineId?: string,
  ) => void;
  onSelectPreviewResult: (runEngineId: string) => void;
  onSelectRegion: (pageNo: number, regionId: string) => void;
  onStart: (options?: {
    embeddingAfterIngest?: boolean;
    embeddingDimension?: number;
    embeddingModelId?: string;
    engines?: IngestEngineSelection[];
    reprocess?: boolean;
    textIndexAfterIngest?: boolean;
  }) => void;
  onStartTextIndex: (sourceRunId: string) => void;
  onGenerateEmbedding: (input: {
    dimension?: number;
    modelId: string;
    sourceRunId: string;
  }) => void;
  onSearchRouteSearchChange: (patch: Partial<SearchRouteSearch>) => void;
  onStop: (runId?: string) => void;
  onThemeChange: (theme: ThemeMode) => void;
}

export function WorkbenchViewContent(props: WorkbenchViewContentProps) {
  switch (props.activeView) {
    case 'models':
      return renderModelView(props);
    case 'diagnostics':
      return renderDiagnosticsView(props);
    case 'ingest':
      return renderIngestView(props);
    case 'search':
      return renderSearchView(props);
    case 'settings':
      return renderSettingsView(props);
    default:
      return renderWorkbenchView(props);
  }
}

function renderModelView(props: WorkbenchViewContentProps) {
  if (props.modelDetailId) {
    return (
      <ModelDetailPanel
        busy={props.modelBusy}
        model={props.models?.models.find((model) => model.model_id === props.modelDetailId)}
        onCancelModel={props.onCancelModel}
        onDownloadModel={props.onDownloadModel}
        onSelectModel={props.onSelectModel}
      />
    );
  }
  return (
    <ModelManager
      busy={props.modelBusy}
      models={props.models}
      onCancelModel={props.onCancelModel}
      onDownloadModel={props.onDownloadModel}
      onRouteSearchChange={props.onModelRouteSearchChange}
      onScopeChange={props.onModelScopeChange}
      onSelectModel={props.onSelectModel}
      routeSearch={props.modelSearch}
      scope={props.modelScope}
      status={props.status}
    />
  );
}

function renderDiagnosticsView(props: WorkbenchViewContentProps) {
  return (
    <DiagnosticsPanel
      activeRunId={props.activeRunId}
      logs={props.logs}
      workUnitId={props.diagnosticWorkUnitId}
      onResumeRun={props.onResumeRun}
      onRestartRun={props.onRestartRun}
      runs={props.runs}
      search={props.diagnosticsSearch}
      onSearchChange={props.onDiagnosticsSearchChange}
      onStopRun={props.onStop}
    />
  );
}

function renderIngestView(props: WorkbenchViewContentProps) {
  return (
    <IngestStartPanel
      activeRun={props.activeRun}
      activeRunId={props.activeRunId}
      busy={props.ingestBusy}
      folderDialogError={props.folderDialogError}
      ingestSearch={props.ingestSearch}
      enginePresets={props.enginePresets}
      model={props.model}
      models={props.models}
      onCancelModel={props.onCancelModel}
      onDownloadModel={props.onDownloadModel}
      onModelChange={props.onModelChange}
      onPickFolder={props.onPickFolder}
      onProfileChange={props.onProfileChange}
      onRootPathChange={props.onRootPathChange}
      onGenerateEmbedding={props.onGenerateEmbedding}
      onStart={props.onStart}
      onStartTextIndex={props.onStartTextIndex}
      onStop={props.onStop}
      profiles={props.profiles}
      rootPath={props.rootPath}
      runs={props.runs}
      selectedProfile={props.selectedProfile}
      status={props.status}
    />
  );
}

function renderSearchView(props: WorkbenchViewContentProps) {
  return (
    <SearchView
      activeRunId={props.activeRunId}
      documents={props.documents}
      logs={props.logs}
      model={props.model}
      onAutoFollowChange={props.onAutoFollowChange}
      onSelectPreviewResult={props.onSelectPreviewResult}
      onOpenModels={props.onOpenModels}
      onPickFolder={props.onPickFolder}
      diagnosticWorkUnits={props.diagnosticWorkUnits}
      pipelineTasks={props.pipelineTasks}
      onResumeRun={props.onResumeRun}
      onRestartRun={props.onRestartRun}
      onRouteSearchChange={props.onSearchRouteSearchChange}
      onStopRun={props.onStop}
      previewPages={props.previewPages}
      previewResults={props.previewResults}
      regions={props.regions}
      rootPath={props.rootPath}
      runs={props.runs}
      search={props.searchSearch}
      selectedDocument={props.selectedDocument}
      selectedRunEngineId={props.selectedRunEngineId}
      textPages={props.textPages}
      workbench={props.workbench}
    />
  );
}

function renderSettingsView(props: WorkbenchViewContentProps) {
  return (
    <SettingsPanel
      activeSection={props.settingsSearch?.section}
      busy={props.settingsBusy}
      models={props.models}
      onDownloadConcurrencyChange={props.onDownloadConcurrencyChange}
      onModelChange={props.onModelChange}
      onProfileChange={props.onProfileChange}
      onRuntimeChange={props.onRuntimeChange}
      onThemeChange={props.onThemeChange}
      profiles={props.profiles}
      selectedProfile={props.selectedProfile}
      settings={props.settings}
      theme={props.theme}
    />
  );
}

function renderWorkbenchView(props: WorkbenchViewContentProps) {
  return (
    <WorkbenchPanels
      activeRunId={props.activeRunId}
      documents={props.documents}
      explorerFilter={props.explorerFilter}
      logs={props.logs}
      model={props.model}
      onAutoFollowChange={props.onAutoFollowChange}
      onExplorerFilterChange={props.onExplorerFilterChange}
      onOpenModels={props.onOpenModels}
      onPickFolder={props.onPickFolder}
      diagnosticWorkUnits={props.diagnosticWorkUnits}
      onResumeRun={props.onResumeRun}
      onRestartRun={props.onRestartRun}
      onSelectDocument={props.onSelectDocument}
      onSelectPreviewResult={props.onSelectPreviewResult}
      onSelectRegion={props.onSelectRegion}
      onStopRun={props.onStop}
      onStart={props.onOpenIngest}
      pipelineTasks={props.pipelineTasks}
      previewPages={props.previewPages}
      previewResults={props.previewResults}
      regions={props.regions}
      rootPath={props.rootPath}
      runs={props.runs}
      selectedDocument={props.selectedDocument}
      selectedRunEngineId={props.selectedRunEngineId}
      textPages={props.textPages}
      workbench={props.workbench}
    />
  );
}
