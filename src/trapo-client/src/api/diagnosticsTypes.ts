export interface RealtimeEventRecord {
  sequence: number;
  type: string;
  occurred_at: string;
  run_id?: string | null;
  file_hash?: string | null;
  page_no?: number | null;
  payload: Record<string, unknown>;
}

export interface OcrReplayPayload {
  events: RealtimeEventRecord[];
  next_since_sequence?: number | null;
}

export interface DiagnosticRunRecord {
  run_id: string;
  root_path: string;
  status: string;
  started_at?: string | null;
  finished_at?: string | null;
  duration_ms: number;
  span_count: number;
  error_count: number;
  file_count: number;
  page_count: number;
}

export interface DiagnosticRunsPayload {
  runs: DiagnosticRunRecord[];
}

export interface DiagnosticSpanRecord {
  span_id: string;
  trace_id: string;
  parent_span_id?: string | null;
  task_id?: string | null;
  work_unit_id?: string | null;
  span_kind: string;
  run_id?: string | null;
  file_hash?: string | null;
  page_no?: number | null;
  name: string;
  pipeline_step: string;
  category: string;
  annotation_engine?: string | null;
  status: string;
  started_at: string;
  ended_at: string;
  duration_ms: number;
  attributes: Record<string, unknown>;
  error_type?: string | null;
  error_message?: string | null;
  error_stack?: string | null;
}

export interface DiagnosticEventRecord {
  event_id: string;
  trace_id: string;
  span_id?: string | null;
  run_id?: string | null;
  file_hash?: string | null;
  page_no?: number | null;
  timestamp: string;
  event_type: string;
  name: string;
  severity: string;
  message: string;
  attributes: Record<string, unknown>;
}

export interface DiagnosticTraceSummary {
  run_id?: string | null;
  span_count: number;
  event_count: number;
  error_count: number;
  total_duration_ms: number;
}

export interface DiagnosticTracePayload {
  summary: DiagnosticTraceSummary;
  spans: DiagnosticSpanRecord[];
  events: DiagnosticEventRecord[];
}

export interface DiagnosticWaterfallRowRecord {
  row_id: string;
  trace_id: string;
  parent_row_id?: string | null;
  span_id?: string | null;
  task_id?: string | null;
  work_unit_id?: string | null;
  run_id?: string | null;
  file_hash?: string | null;
  filename?: string | null;
  page_no?: number | null;
  label: string;
  row_source: 'pipeline_task' | 'diagnostic_span' | 'work_unit' | string;
  pipeline_step: string;
  category: string;
  span_kind: string;
  status: string;
  started_at?: string | null;
  ended_at?: string | null;
  duration_ms: number;
  start_ms?: number | null;
  end_ms?: number | null;
  visual_start_ms?: number | null;
  visual_end_ms?: number | null;
  visual_duration_ms: number;
  depth: number;
  child_count: number;
  sort_index: number;
  attributes: Record<string, unknown>;
  error_type?: string | null;
  error_message?: string | null;
}

export interface DiagnosticWaterfallSummary {
  run_id?: string | null;
  row_count: number;
  trace_count: number;
  error_count: number;
  start_ms?: number | null;
  end_ms?: number | null;
  duration_ms: number;
}

export interface DiagnosticWaterfallPayload {
  summary: DiagnosticWaterfallSummary;
  rows: DiagnosticWaterfallRowRecord[];
}

export interface DiagnosticWorkUnitRecord {
  work_unit_id: string;
  run_id: string;
  work_key: string;
  file_hash?: string | null;
  filename?: string | null;
  source_path?: string | null;
  page_no?: number | null;
  phase: string;
  engine: string;
  provider: string;
  model: string;
  profile?: string | null;
  execution_key: string;
  artifact_variant?: string | null;
  status: string;
  attempt_count: number;
  queued_at: string;
  started_at?: string | null;
  finished_at?: string | null;
  duration_ms?: number | null;
  error?: string | null;
  result: Record<string, unknown>;
  metadata: Record<string, unknown>;
}

export interface DiagnosticModelLeaseRecord {
  lease_id: string;
  run_id: string;
  execution_key: string;
  provider: string;
  model: string;
  requested_context_tokens?: number | null;
  verified_context_tokens?: number | null;
  status: string;
  started_at: string;
  finished_at?: string | null;
  duration_ms?: number | null;
  error?: string | null;
  metadata: Record<string, unknown>;
}

export interface DiagnosticProgressSummary {
  total_work_units: number;
  queued: number;
  running: number;
  completed: number;
  failed: number;
  cancelled: number;
}

export interface DiagnosticPipelineTaskRecord {
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

export interface DiagnosticProgressPayload {
  summary: DiagnosticProgressSummary;
  work_units: DiagnosticWorkUnitRecord[];
  model_leases: DiagnosticModelLeaseRecord[];
  pipeline_tasks: DiagnosticPipelineTaskRecord[];
}

export interface DiagnosticBreakdownRecord {
  key: string;
  count: number;
  total_duration_ms: number;
}

export interface DiagnosticSlowSpanRecord {
  span_id: string;
  name: string;
  pipeline_step: string;
  duration_ms: number;
  status: string;
}

export interface DiagnosticRecommendationRecord {
  severity: string;
  title: string;
  detail: string;
}

export interface DiagnosticAnalyticsSummary {
  span_count: number;
  event_count: number;
  error_count: number;
  total_duration_ms: number;
  average_span_ms: number;
}

export interface DiagnosticAnalyticsPayload {
  summary: DiagnosticAnalyticsSummary;
  by_pipeline_step: DiagnosticBreakdownRecord[];
  by_category: DiagnosticBreakdownRecord[];
  slow_spans: DiagnosticSlowSpanRecord[];
  recommendations: DiagnosticRecommendationRecord[];
}

export interface DiagnosticModelsPayload {
  model_leases: DiagnosticModelLeaseRecord[];
}
