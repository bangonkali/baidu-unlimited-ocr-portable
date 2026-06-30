export type RunState =
  | 'idle'
  | 'queued'
  | 'running'
  | 'cancelled'
  | 'failed'
  | 'completed'
  | 'completed_with_errors';

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
  inference_engine?: string;
  log_path?: string;
  database_path?: string;
  realtime_path?: string;
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
}

export interface ModelDownloadRecord {
  model_id: string;
  status: string;
}

export interface ModelDownloadRequest {
  force?: boolean;
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
  content_markdown?: string;
  content_html?: string | null;
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
  error?: string;
}

export interface AnnotationSettingsPayload {
  show_boxes: boolean;
  show_labels: boolean;
  box_color: string;
  active_box_color: string;
}

export interface PreviewImagesPayload {
  file_hash: string;
  variants: string[];
  pages: number[];
}

export interface LogRecord {
  timestamp: string;
  level: string;
  component: string;
  message: string;
}

export interface LogsPayload {
  log_path?: string;
  logs: LogRecord[];
}
