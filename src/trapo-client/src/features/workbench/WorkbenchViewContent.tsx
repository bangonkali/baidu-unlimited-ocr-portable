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
import type { ActiveView, ThemeMode, useWorkbenchState } from '../../stores/workbenchStore';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { IngestStartPanel } from './IngestStartPanel';
import { ModelDetailPanel } from './ModelDetailPanel';
import { ModelManager } from './ModelManager';
import { SettingsPanel } from './SettingsPanel';
import { WorkbenchPanels } from './WorkbenchPanels';

export interface WorkbenchViewContentProps {
  activeView: ActiveView;
  activeRun?: IngestRunRecord;
  activeRunId?: string | null;
  diagnosticsSearch?: DiagnosticsRouteSearch;
  documents: DocumentSummary[];
  folderDialogError?: string;
  ingestBusy: boolean;
  ingestSearch?: IngestRouteSearch;
  logs: LogRecord[];
  model?: ModelAssetRecord;
  modelBusy: boolean;
  modelDetailId?: string;
  modelScope: 'library' | 'downloads';
  modelSearch?: ModelRouteSearch;
  models?: ModelsPayload;
  previewPages: number[];
  profiles: OcrProfileRecord[];
  regions: OverlayBox[];
  rootPath: string;
  runs: IngestRunRecord[];
  selectedDocument?: DocumentSummary;
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
  onSelectDocument: (fileHash: string, pageNo?: number) => void;
  onSelectRegion: (pageNo: number, regionId: string) => void;
  onStart: (options?: { reprocess?: boolean }) => void;
  onStop: (runId?: string) => void;
  onThemeChange: (theme: ThemeMode) => void;
}

export function WorkbenchViewContent(props: WorkbenchViewContentProps) {
  if (props.activeView === 'models') {
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
  if (props.activeView === 'diagnostics') {
    return (
      <DiagnosticsPanel
        activeRunId={props.activeRunId}
        logs={props.logs}
        onResumeRun={props.onResumeRun}
        onRestartRun={props.onRestartRun}
        runs={props.runs}
        search={props.diagnosticsSearch}
        onSearchChange={props.onDiagnosticsSearchChange}
        onStopRun={props.onStop}
      />
    );
  }
  if (props.activeView === 'ingest') {
    return (
      <IngestStartPanel
        activeRun={props.activeRun}
        activeRunId={props.activeRunId}
        busy={props.ingestBusy}
        folderDialogError={props.folderDialogError}
        ingestSearch={props.ingestSearch}
        model={props.model}
        models={props.models}
        onModelChange={props.onModelChange}
        onPickFolder={props.onPickFolder}
        onProfileChange={props.onProfileChange}
        onRootPathChange={props.onRootPathChange}
        onStart={props.onStart}
        onStop={props.onStop}
        profiles={props.profiles}
        rootPath={props.rootPath}
        selectedProfile={props.selectedProfile}
        status={props.status}
      />
    );
  }
  if (props.activeView === 'settings') {
    return (
      <SettingsPanel
        activeSection={props.settingsSearch?.section}
        busy={props.settingsBusy}
        models={props.models}
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
  return (
    <WorkbenchPanels
      activeRunId={props.activeRunId}
      documents={props.documents}
      logs={props.logs}
      model={props.model}
      onAutoFollowChange={props.onAutoFollowChange}
      onOpenModels={props.onOpenModels}
      onPickFolder={props.onPickFolder}
      onResumeRun={props.onResumeRun}
      onRestartRun={props.onRestartRun}
      onSelectDocument={props.onSelectDocument}
      onSelectRegion={props.onSelectRegion}
      onStopRun={props.onStop}
      onStart={props.onOpenIngest}
      previewPages={props.previewPages}
      regions={props.regions}
      rootPath={props.rootPath}
      runs={props.runs}
      selectedDocument={props.selectedDocument}
      textPages={props.textPages}
      workbench={props.workbench}
    />
  );
}
