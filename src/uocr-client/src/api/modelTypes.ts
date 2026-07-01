export interface OcrProfileRecord {
  key: string;
  label: string;
  engine_name: string;
  description: string;
  default_max_tokens: number;
  ngram_size?: number;
  ngram_window?: number;
  pdf_ngram_window?: number;
  force_prompt_eos?: boolean;
  no_image_end?: boolean;
}

export interface ModelAssetRecord {
  model_id: string;
  display_name: string;
  status: string;
  repo_id?: string;
  revision?: string;
  local_path?: string | null;
  size_bytes?: number | null;
  error?: string | null;
  model_file?: string;
  mmproj_file?: string;
  current_file?: string | null;
  status_message?: string | null;
  downloaded_bytes?: number;
  total_bytes?: number | null;
  overall_downloaded_bytes?: number;
  overall_total_bytes?: number | null;
  overall_percent?: number;
  bytes_per_second?: number;
  eta_seconds?: number | null;
  auth_available?: boolean;
  auth_source?: string | null;
  last_event_at?: string | null;
  files?: ModelDownloadFileRecord[];
  quantization?: string;
  bits?: number;
  quality?: string;
  hardware_tier?: string;
  notes?: string;
  recommended?: boolean;
  selected?: boolean;
  provider_name?: string;
  total_required_bytes?: number | null;
  downloaded_file_count?: number;
  total_file_count?: number;
}

export interface ModelDownloadFileRecord {
  file_id: string;
  file_name: string;
  status: string;
  local_path?: string | null;
  downloaded_bytes: number;
  total_bytes?: number | null;
  percent: number;
  bytes_per_second?: number;
  eta_seconds?: number | null;
  error?: string | null;
}

export interface ModelsPayload {
  models: ModelAssetRecord[];
  profiles: OcrProfileRecord[];
  selected_model_id?: string;
  provider_repo?: string;
  provider_label?: string;
  shared_mmproj_file?: string;
}

export interface ModelDownloadRecord {
  model_id: string;
  status: string;
}

export interface ModelSelectRecord {
  model_id: string;
  status: string;
}

export interface ModelDownloadRequest {
  force?: boolean;
}
