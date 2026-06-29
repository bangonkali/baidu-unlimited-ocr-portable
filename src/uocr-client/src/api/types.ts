export type RunState =
  | 'idle'
  | 'queued'
  | 'running'
  | 'paused'
  | 'cancelled'
  | 'failed'
  | 'completed';

export interface StatusPayload {
  state: RunState | string;
  host?: string;
  active_run_id?: string | null;
  default_profile: string;
  version?: string;
  git_tag?: string;
  git_sha?: string;
  supported_inputs: string[];
}

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
  local_path?: string | null;
  size_bytes?: number | null;
  error?: string | null;
  model_file?: string;
  mmproj_file?: string;
}

export interface ModelsPayload {
  models: ModelAssetRecord[];
  profiles: OcrProfileRecord[];
}

export interface ModelDownloadRecord {
  model_id: string;
  status: string;
}

export interface SettingsPayload {
  pdf_dpi: number;
  ocr_concurrency: number;
  default_profile: string;
  retry_profile: string;
  cache_path?: string;
  database_path?: string;
}

export interface IngestStartRequest {
  root_path: string;
  profile_id?: string;
  engine_id?: string;
  reprocess?: boolean;
}

export interface IngestRunRecord {
  run_id: string;
  root_path: string;
  status: RunState | string;
  queued_files?: number;
  processed_pages?: number;
  total_pages?: number;
  error?: string | null;
}

export interface IngestRunsPayload {
  runs: IngestRunRecord[];
}

export interface DocumentSummary {
  file_hash: string;
  display_name: string;
  relative_path?: string;
  status: string;
  page_count: number;
  regions?: number;
  error?: string;
}

export interface DocumentsPayload {
  documents: DocumentSummary[];
}

export interface DocumentRegionsPayload {
  file_hash: string;
  boxes: OverlayBox[];
}

export interface OverlayBox {
  region_id: string;
  label: string;
  page_no: number;
  left_percent: number;
  top_percent: number;
  width_percent: number;
  height_percent: number;
  hidden?: boolean;
}

export interface TextRegionSpan {
  region_id: string;
  page_no: number;
  start: number;
  end: number;
}

export interface PageTextRecord {
  page_no: number;
  text: string;
  spans: TextRegionSpan[];
}

export interface DocumentTextPayload {
  file_hash: string;
  pages: PageTextRecord[];
}

export interface FolderDialogResponse {
  cancelled: boolean;
  selected_path: string;
  manual_path_supported: boolean;
}

export interface AnnotationSettingsPayload {
  show_boxes: boolean;
  show_labels: boolean;
  box_color: string;
  active_box_color: string;
}
