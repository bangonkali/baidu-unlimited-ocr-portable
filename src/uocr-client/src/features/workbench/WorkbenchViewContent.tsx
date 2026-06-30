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
import type { ModelRouteSearch, SettingsRouteSearch } from '../../routeSearch';
import type { ActiveView, useWorkbenchState } from '../../stores/workbenchStore';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { ModelManager } from './ModelManager';
import { SettingsPanel } from './SettingsPanel';
import { WorkbenchPanels } from './WorkbenchPanels';

interface WorkbenchViewContentProps {
  activeView: ActiveView;
  documents: DocumentSummary[];
  logs: LogRecord[];
  model?: ModelAssetRecord;
  modelBusy: boolean;
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
  textPages: PageTextRecord[];
  workbench: ReturnType<typeof useWorkbenchState>;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
  onModelChange: (modelId: string) => void;
  onModelRouteSearchChange: (patch: Partial<ModelRouteSearch>) => void;
  onModelScopeChange: (scope: 'library' | 'downloads') => void;
  onOpenModels: () => void;
  onPickFolder: () => void;
  onProfileChange: (profileId: string) => void;
  onRuntimeChange: (runtimeId: string) => void;
  onSelectModel: (modelId: string) => void;
  onStart: () => void;
}

export function WorkbenchViewContent(props: WorkbenchViewContentProps) {
  if (props.activeView === 'models') {
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
    return <DiagnosticsPanel logs={props.logs} runs={props.runs} />;
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
        profiles={props.profiles}
        selectedProfile={props.selectedProfile}
        settings={props.settings}
      />
    );
  }
  return (
    <WorkbenchPanels
      documents={props.documents}
      logs={props.logs}
      model={props.model}
      onOpenModels={props.onOpenModels}
      onPickFolder={props.onPickFolder}
      onStart={props.onStart}
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
