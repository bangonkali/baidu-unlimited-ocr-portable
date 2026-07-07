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
  runtime_selectable?: boolean;
  runtime_variants?: RuntimeVariantRecord[];
  inference_engine?: string;
  log_path?: string;
  database_path?: string;
  realtime_path?: string;
  selected_model_id?: string;
  duckdb_extensions?: DuckDbExtensionsRecord;
}

export interface DuckDbExtensionsRecord {
  fts_loaded: boolean;
  fts_error?: string | null;
  vss_loaded: boolean;
  vss_error?: string | null;
  duckpgq_loaded: boolean;
  duckpgq_error?: string | null;
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
  model_kind?: 'ocr' | 'embedding' | string;
  routing_origin?: 'unlimited_ocr' | 'embedding' | string;
  status: string;
  repo_id?: string;
  revision?: string;
  local_path?: string | null;
  size_bytes?: number | null;
  error?: string | null;
  error_kind?: string | null;
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
  embedding_dimension?: number | null;
  context_tokens?: number | null;
  pooling?: string | null;
  normalize_embeddings?: boolean | null;
  query_prefix?: string | null;
  document_prefix?: string | null;
  recommended_vram_gb?: number | null;
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
  error_kind?: string | null;
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

export interface SettingsPayload {
  pdf_dpi: number;
  ocr_concurrency: number;
  download_concurrency: number;
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
  download_concurrency?: number;
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

export interface IngestStartRequest {
  root_path: string;
  profile_id?: string;
  model_id?: string;
  runtime_id?: string;
  engine_id?: string;
  engines?: IngestEngineSelection[];
  reprocess?: boolean;
  text_index_after_ingest?: boolean;
  embedding_after_ingest?: boolean;
  embedding_model_id?: string;
  embedding_dimension?: number;
}

export interface IngestEngineSelection {
  preset_id?: string;
  engine_id: string;
  engine_kind: string;
  model_id?: string | null;
  profile_id?: string | null;
  runtime_id?: string | null;
  parameters?: Record<string, unknown>;
  ordinal?: number;
}

export interface IngestEngineConfigRecord {
  run_engine_id: string;
  run_id: string;
  ordinal: number;
  engine_kind: string;
  engine_id: string;
  label: string;
  model_id?: string | null;
  profile_id?: string | null;
  runtime_id?: string | null;
  parameters: Record<string, unknown>;
  status: string;
  error?: string | null;
  usable_output_count: number;
  previewer: string;
}

export interface IngestEnginePresetRecord {
  preset_id: string;
  engine_id: string;
  engine_kind: string;
  label: string;
  description: string;
  model_id?: string | null;
  profile_id?: string | null;
  runtime_id?: string | null;
  previewer: string;
  default_enabled: boolean;
  requires_model: boolean;
  download_model_ids: string[];
  available: boolean;
  availability: string;
  availability_detail?: string | null;
  runner_kind: string;
  runner_status: string;
  runner_detail?: string | null;
  parameter_schema: Record<string, unknown>;
  default_parameters: Record<string, unknown>;
}

export interface IngestEnginesPayload {
  engines: IngestEnginePresetRecord[];
}

export interface IngestPreviewResultRecord {
  run_engine_id: string;
  run_id: string;
  ordinal: number;
  engine_kind: string;
  engine_id: string;
  label: string;
  model_id?: string | null;
  profile_id?: string | null;
  runtime_id?: string | null;
  status: string;
  previewer: string;
  output_count: number;
  page_count: number;
  error?: string | null;
  runner_kind: string;
  runner_status: string;
  runner_detail?: string | null;
  provenance: Record<string, unknown>;
}

export interface IngestPreviewResultsPayload {
  run_id: string;
  file_hash: string;
  results: IngestPreviewResultRecord[];
}

export interface PipelineTaskRecord {
  task_id: string;
  task_kind: string;
  origin_run_id?: string | null;
  status: string;
  params: Record<string, unknown>;
  result: Record<string, unknown>;
  queued_at: string;
  started_at?: string | null;
  finished_at?: string | null;
  runner_id?: string | null;
  error?: string | null;
}

export interface TextIndexRequest {
  source_run_id: string;
}

export interface TextIndexResponse {
  task: PipelineTaskRecord;
  text_index_run_id: string;
  source_run_id: string;
  segments_indexed: number;
  status: string;
}

export interface GenerateEmbeddingRequest {
  source_run_id: string;
  model_id: string;
  dimension?: number;
}

export interface GenerateEmbeddingResponse {
  task: PipelineTaskRecord;
  embedding_run_id: string;
  source_run_id: string;
  model_id: string;
  dimension: number;
  segments_embedded: number;
  status: string;
}

export interface UsedEmbeddingModelRecord {
  model_id: string;
  display_name: string;
  provider: string;
  dimension: number;
}

export interface UsedEmbeddingModelsPayload {
  models: UsedEmbeddingModelRecord[];
}

export interface HybridSearchRequest {
  query: string;
  source_run_id?: string;
  embedding_model_id?: string;
  limit?: number;
}

export interface HybridSearchHit {
  segment_id: string;
  file_hash: string;
  page_no: number;
  annotation_id?: string | null;
  category: string;
  text: string;
  score: number;
  relevance_score: number;
  rank: number;
  hit_source: string;
  model_id?: string | null;
}

export interface HybridSearchFileResult {
  file_hash: string;
  hit_count: number;
  relevance_score: number;
  hits: HybridSearchHit[];
}

export interface HybridSearchResponse {
  query: string;
  hits: HybridSearchHit[];
  files: HybridSearchFileResult[];
}

export interface RunCompletionManifestRecord {
  run_id: string;
  completed_at: string;
  status: string;
  root_path: string;
  profile_id: string;
  engine_id: string;
  model_id: string;
  runtime_id: string;
  queued_files: number;
  processed_pages: number;
  total_pages: number;
  file_count: number;
  page_count: number;
  summary: Record<string, unknown>;
}

export interface IngestRunRecord {
  run_id: string;
  root_path: string;
  status: RunState | string;
  file_hashes?: string[];
  queued_files?: number;
  processed_pages?: number;
  total_pages?: number;
  current_page?: number;
  progress_percent?: number;
  profile_id?: string;
  engine_id?: string;
  model_id?: string;
  runtime_id?: string;
  error?: string | null;
  can_resume?: boolean;
  can_restart?: boolean;
  engine_configs?: IngestEngineConfigRecord[];
  preview_results?: IngestPreviewResultRecord[];
  completion_manifest?: RunCompletionManifestRecord | null;
}

export interface IngestStartResponse {
  run: IngestRunRecord;
  documents: DocumentSummary[];
  replay_since_sequence: number;
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
  processed_pages?: number;
  total_pages?: number;
  current_page?: number;
  progress_percent?: number;
  regions?: number;
  error?: string;
}

export interface DocumentsPayload {
  documents: DocumentSummary[];
}

export interface DocumentRegionsPayload {
  file_hash: string;
  run_id?: string | null;
  run_engine_id?: string | null;
  boxes: OverlayBox[];
}

export interface OverlayBox {
  annotation_id?: string;
  region_id: string;
  label: string;
  category?: string;
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
  annotation_id?: string;
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
  run_id?: string | null;
  run_engine_id?: string | null;
  pages: PageTextRecord[];
}

export type * from './diagnosticsTypes';
export type * from './systemTypes';
export type * from './workbenchAuxTypes';
