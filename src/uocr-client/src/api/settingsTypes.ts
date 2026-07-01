import type { RuntimeVariantRecord } from './runtimeTypes';

export interface SettingsPayload {
  pdf_dpi: number;
  ocr_concurrency: number;
  default_profile: string;
  retry_profile: string;
  cache_path?: string;
  database_path?: string;
  selected_runtime_id?: string;
  selected_accelerator?: string;
  selected_model_id?: string;
  runtime_variants?: RuntimeVariantRecord[];
  workbench_ui?: WorkbenchUiSettings;
}

export interface SettingsUpdateRequest {
  default_profile?: string;
  selected_runtime_id?: string;
  workbench_ui?: WorkbenchUiSettingsPatch;
}

export type WorkbenchThemeMode = 'dark' | 'light';

export interface WorkbenchPaneSettings {
  details: boolean;
  diagnostics: boolean;
  explorer: boolean;
}

export interface WorkbenchPaneSettingsPatch {
  details?: boolean;
  diagnostics?: boolean;
  explorer?: boolean;
}

export interface WorkbenchUiSettings {
  auto_follow_regions: boolean;
  labels_visible: boolean;
  overlay_visible: boolean;
  panes_collapsed: WorkbenchPaneSettings;
  theme: WorkbenchThemeMode;
}

export interface WorkbenchUiSettingsPatch {
  auto_follow_regions?: boolean;
  labels_visible?: boolean;
  overlay_visible?: boolean;
  panes_collapsed?: WorkbenchPaneSettingsPatch;
  theme?: WorkbenchThemeMode;
}
