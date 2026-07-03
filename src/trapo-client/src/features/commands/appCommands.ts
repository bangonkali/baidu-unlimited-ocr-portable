import type { ModelAssetRecord, ModelsPayload, StatusPayload } from '../../api/types';
import type {
  DiagnosticsRouteSearch,
  IngestRouteSearch,
  ModelRouteSearch,
  SettingsRouteSearch,
  WorkbenchRouteSearch,
} from '../../routeSearch';
import type { ThemeMode, WorkbenchPaneId, WorkbenchPaneState } from '../../stores/workbenchStore';

export type CommandIconName =
  | 'diagnostics'
  | 'download'
  | 'folder'
  | 'layout'
  | 'model'
  | 'route'
  | 'settings'
  | 'theme';

export type CommandRouteTarget =
  | { to: '/diagnostics'; search?: DiagnosticsRouteSearch }
  | { to: '/ingest/start'; search?: IngestRouteSearch }
  | { to: '/models'; search?: ModelRouteSearch }
  | { to: '/models/$modelId'; params: { modelId: string }; search?: ModelRouteSearch }
  | { to: '/models/downloads'; search?: ModelRouteSearch }
  | { to: '/settings'; search?: SettingsRouteSearch }
  | { to: '/workbench'; search?: WorkbenchRouteSearch };

export type AppCommandAction =
  | { kind: 'cancelModelDownload'; modelId: string }
  | { kind: 'downloadModel'; force?: boolean; modelId: string }
  | { kind: 'filterDocuments'; q: string }
  | { kind: 'navigate'; target: CommandRouteTarget }
  | { kind: 'openGuide' }
  | { kind: 'selectModel'; modelId: string }
  | { kind: 'setTheme'; theme: ThemeMode }
  | { kind: 'togglePane'; pane: WorkbenchPaneId };

export interface AppCommand {
  action: AppCommandAction;
  description: string;
  disabled?: boolean;
  group: string;
  icon: CommandIconName;
  id: string;
  keywords: readonly string[];
  label: string;
  shortcut?: string;
}

interface BuildAppCommandsArgs {
  models?: ModelsPayload;
  panesCollapsed: WorkbenchPaneState;
  status?: StatusPayload;
  theme: ThemeMode;
}

export function buildAppCommands({
  models,
  panesCollapsed,
  status,
  theme,
}: BuildAppCommandsArgs): AppCommand[] {
  return [
    ...navigationCommands(),
    ...settingsCommands(theme),
    ...paneCommands(panesCollapsed),
    ...ingestCommands(status),
    ...modelCommands(models?.models ?? []),
    {
      action: { kind: 'openGuide' },
      description: 'Restart the guided workbench tour.',
      group: 'Help',
      icon: 'route',
      id: 'help.startGuide',
      keywords: ['tour', 'help', 'onboarding'],
      label: 'Start guide',
    },
  ];
}

export function isActiveIngestState(status?: StatusPayload) {
  return Boolean(status?.active_run_id) && ['queued', 'running'].includes(String(status?.state));
}

function navigationCommands(): AppCommand[] {
  return [
    nav('nav.workbench', 'Workbench', 'Open OCR preview, text, explorer, and details.', {
      to: '/workbench',
    }),
    nav('nav.models', 'Model Library', 'Browse all available Unlimited-OCR model variants.', {
      to: '/models',
    }),
    nav('nav.downloads', 'Active Downloads', 'Show queued and in-progress file downloads.', {
      search: { status: 'all' },
      to: '/models/downloads',
    }),
    nav('nav.ingest', 'Start Ingest', 'Configure a folder scan and OCR run.', {
      to: '/ingest/start',
    }),
    nav('nav.diagnostics', 'Diagnostics', 'Open OCR waterfall, progress, and logs.', {
      search: { tab: 'waterfall' },
      to: '/diagnostics',
    }),
    nav('nav.settings', 'Settings', 'Open application settings.', {
      to: '/settings',
    }),
  ];
}

function settingsCommands(theme: ThemeMode): AppCommand[] {
  return [
    nav('settings.appearance', 'Settings: Appearance', 'Theme and visual preferences.', {
      search: { section: 'appearance' },
      to: '/settings',
    }),
    nav('settings.runtime', 'Settings: Runtime', 'CPU and CUDA runtime selection.', {
      search: { section: 'runtime' },
      to: '/settings',
    }),
    nav('settings.ocr', 'Settings: OCR Defaults', 'Default model, profile, DPI, and concurrency.', {
      search: { section: 'ocr' },
      to: '/settings',
    }),
    {
      action: { kind: 'setTheme', theme: theme === 'dark' ? 'light' : 'dark' },
      description: `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme.`,
      group: 'Settings',
      icon: 'theme',
      id: 'theme.toggle',
      keywords: ['appearance', 'color', 'theme'],
      label: theme === 'dark' ? 'Switch to Light Theme' : 'Switch to Dark Theme',
    },
  ];
}

function paneCommands(panesCollapsed: WorkbenchPaneState): AppCommand[] {
  return (['explorer', 'diagnostics', 'details'] as const).map((pane) => ({
    action: { kind: 'togglePane', pane },
    description: `${panesCollapsed[pane] ? 'Show' : 'Hide'} the ${pane} pane.`,
    group: 'Layout',
    icon: 'layout',
    id: `layout.toggle.${pane}`,
    keywords: ['panel', 'pane', 'layout', pane],
    label: `${panesCollapsed[pane] ? 'Show' : 'Hide'} ${paneLabel(pane)}`,
  }));
}

function ingestCommands(status?: StatusPayload): AppCommand[] {
  const active = isActiveIngestState(status);
  return [
    {
      action: { kind: 'navigate', target: { to: '/ingest/start' } },
      description: active
        ? 'An ingest is already active; open its status.'
        : 'Choose a folder, model, and OCR profile.',
      disabled: false,
      group: 'Ingest',
      icon: 'folder',
      id: 'ingest.configure',
      keywords: ['scan', 'folder', 'ocr', 'start'],
      label: active ? 'View Active Ingest' : 'Configure Start Ingest',
    },
  ];
}

function modelCommands(models: ModelAssetRecord[]): AppCommand[] {
  return models.flatMap((model) => {
    const commands: AppCommand[] = [
      {
        action: {
          kind: 'navigate',
          target: { params: { modelId: model.model_id }, to: '/models/$modelId' },
        },
        description: model.status_message ?? model.notes ?? model.status,
        group: 'Models',
        icon: 'model',
        id: `model.open.${model.model_id}`,
        keywords: modelKeywords(model),
        label: `Open ${model.display_name}`,
      },
    ];
    if (model.status === 'downloaded') {
      commands.push({
        action: { force: true, kind: 'downloadModel', modelId: model.model_id },
        description: 'Download the model files again.',
        group: 'Models',
        icon: 'download',
        id: `model.redownload.${model.model_id}`,
        keywords: modelKeywords(model),
        label: `Re-download ${model.display_name}`,
      });
      commands.push({
        action: { kind: 'selectModel', modelId: model.model_id },
        description: model.selected ? 'This model is already selected.' : 'Use this model for OCR.',
        disabled: model.selected,
        group: 'Models',
        icon: 'model',
        id: `model.select.${model.model_id}`,
        keywords: modelKeywords(model),
        label: model.selected ? `${model.display_name} is In Use` : `Use ${model.display_name}`,
      });
      return commands;
    }
    if (['downloading', 'queued', 'cancelling'].includes(model.status)) {
      commands.push({
        action: { kind: 'cancelModelDownload', modelId: model.model_id },
        description: 'Cancel queued or in-progress model files.',
        group: 'Models',
        icon: 'download',
        id: `model.cancel.${model.model_id}`,
        keywords: modelKeywords(model),
        label: `Cancel ${model.display_name} Download`,
      });
      return commands;
    }
    commands.push({
      action: { kind: 'downloadModel', modelId: model.model_id },
      description: model.status_message ?? 'Download required GGUF files.',
      group: 'Models',
      icon: 'download',
      id: `model.download.${model.model_id}`,
      keywords: modelKeywords(model),
      label: `Download ${model.display_name}`,
    });
    return commands;
  });
}

function nav(
  id: string,
  label: string,
  description: string,
  target: CommandRouteTarget,
): AppCommand {
  return {
    action: { kind: 'navigate', target },
    description,
    group: 'Navigation',
    icon: 'route',
    id,
    keywords: [label, description],
    label,
  };
}

function modelKeywords(model: ModelAssetRecord) {
  return [
    model.model_id,
    model.display_name,
    model.quantization ?? '',
    model.hardware_tier ?? '',
    model.repo_id ?? '',
    model.status,
  ];
}

function paneLabel(pane: WorkbenchPaneId) {
  switch (pane) {
    case 'details':
      return 'Details';
    case 'diagnostics':
      return 'Diagnostics';
    case 'explorer':
      return 'Explorer';
  }
}
