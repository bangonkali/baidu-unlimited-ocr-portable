import type { RunState } from './ingestTypes';

export interface StatusPayload {
  state: RunState | string;
  host?: string;
  active_run_id?: string | null;
  default_profile: string;
  version?: string;
  git_tag?: string;
  git_sha?: string;
  supported_inputs: string[];
  runtime_platform?: string;
  accelerator?: string;
  runtime_selectable?: boolean;
  runtime_variants?: RuntimeVariantRecord[];
  inference_engine?: string;
  log_path?: string;
  database_path?: string;
  realtime_path?: string;
  selected_model_id?: string;
}

export interface RuntimeVariantRecord {
  runtime_id: string;
  label: string;
  platform: string;
  accelerator: 'cuda' | 'rocm' | 'metal' | 'cpu' | string;
  backend: string;
  ffi_library?: string;
  installed: boolean;
  hardware_supported: boolean;
  selectable: boolean;
  selected: boolean;
  support_detail?: string;
}
