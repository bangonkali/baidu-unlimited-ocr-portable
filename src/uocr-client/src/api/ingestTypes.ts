export type RunState =
  | 'idle'
  | 'queued'
  | 'running'
  | 'cancelled'
  | 'failed'
  | 'completed'
  | 'completed_with_errors';

export interface IngestStartRequest {
  root_path: string;
  profile_id?: string;
  model_id?: string;
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
  current_page?: number;
  progress_percent?: number;
  profile_id?: string;
  engine_id?: string;
  model_id?: string;
  runtime_id?: string;
  error?: string | null;
}

export interface IngestRunsPayload {
  runs: IngestRunRecord[];
}

export interface OcrPageMetricRecord {
  run_id: string;
  file_hash: string;
  page_no: number;
  engine_id?: string;
  profile_id?: string;
  model_id?: string;
  runtime_id?: string;
  runtime_platform?: string;
  accelerator?: string;
  status: string;
  error?: string | null;
  token_count: number;
  chunk_count: number;
  first_token_latency_ms: number;
  generation_duration_ms: number;
  elapsed_ms: number;
  min_tps: number;
  max_tps: number;
  avg_tps: number;
  started_at?: string;
  first_token_at?: string;
  completed_at?: string;
}

export interface OcrMetricsTreeNode {
  id: string;
  kind: 'run' | 'file' | 'page';
  label: string;
  run_id: string;
  file_hash?: string;
  page_no?: number;
  status: string;
  model_id?: string;
  runtime_id?: string;
  runtime_platform?: string;
  accelerator?: string;
  token_count: number;
  chunk_count: number;
  page_count: number;
  first_token_latency_ms: number;
  generation_duration_ms: number;
  elapsed_ms: number;
  min_tps: number;
  max_tps: number;
  avg_tps: number;
  started_at?: string;
  completed_at?: string;
  error?: string | null;
  children?: OcrMetricsTreeNode[];
}

export interface OcrMetricsTreePayload {
  nodes: OcrMetricsTreeNode[];
}
