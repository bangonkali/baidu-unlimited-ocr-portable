#[derive(Debug, Clone)]
pub(crate) struct StoredRun {
    pub(crate) run_id: String,
    pub(crate) root_path: String,
    pub(crate) status: String,
    pub(crate) profile_id: String,
    pub(crate) engine_id: String,
    pub(crate) model_id: String,
    pub(crate) runtime_id: String,
    pub(crate) queued_files: u32,
    pub(crate) processed_pages: u32,
    pub(crate) total_pages: u32,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredRunCompletionManifest {
    pub(crate) run_id: String,
    pub(crate) completed_at: String,
    pub(crate) status: String,
    pub(crate) root_path: String,
    pub(crate) profile_id: String,
    pub(crate) engine_id: String,
    pub(crate) model_id: String,
    pub(crate) runtime_id: String,
    pub(crate) queued_files: u32,
    pub(crate) processed_pages: u32,
    pub(crate) total_pages: u32,
    pub(crate) file_count: u32,
    pub(crate) page_count: u32,
    pub(crate) summary: Value,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CompletedRunPage {
    pub(crate) file_hash: String,
    pub(crate) page_no: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredDocument {
    pub(crate) file_hash: String,
    pub(crate) display_name: String,
    pub(crate) extension: String,
    pub(crate) size_bytes: u64,
    pub(crate) page_count: u32,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
    pub(crate) root_path: String,
    pub(crate) absolute_path: String,
    pub(crate) relative_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredRunDocument {
    pub(crate) run_id: String,
    pub(crate) file_hash: String,
    pub(crate) ordinal: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredPage {
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: String,
    pub(crate) page_no: u32,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
    pub(crate) render_dpi: u32,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
    pub(crate) preview_path: Option<String>,
    pub(crate) cleaned_text: String,
    pub(crate) raw_text: String,
    pub(crate) boxes: Vec<OverlayBox>,
    pub(crate) spans: Vec<TextRegionSpan>,
}

#[derive(Debug, Clone)]
pub(crate) struct AnnotationIdentityDraft {
    pub(crate) annotation_id: Option<String>,
    pub(crate) run_id: String,
    pub(crate) file_hash: String,
    pub(crate) page_no: u32,
    pub(crate) engine_id: String,
    pub(crate) profile_id: String,
    pub(crate) source_region_key: String,
    pub(crate) discovery_index: u32,
    pub(crate) label: String,
    pub(crate) x1: f64,
    pub(crate) y1: f64,
    pub(crate) x2: f64,
    pub(crate) y2: f64,
    pub(crate) span_start: u64,
    pub(crate) span_end: u64,
    pub(crate) content_markdown: String,
    pub(crate) content_html: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct OcrPageMetrics {
    pub(crate) run_id: String,
    pub(crate) file_hash: String,
    pub(crate) page_no: u32,
    pub(crate) model_id: String,
    pub(crate) runtime_id: String,
    pub(crate) status: String,
    pub(crate) token_count: u64,
    pub(crate) avg_tps: f64,
    pub(crate) elapsed_ms: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredRealtimeEvent {
    pub(crate) event_id: String,
    pub(crate) sequence: u64,
    pub(crate) event_type: String,
    pub(crate) occurred_at: String,
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) payload: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticRunRow {
    pub(crate) run_id: String,
    pub(crate) root_path: String,
    pub(crate) status: String,
    pub(crate) started_at: Option<String>,
    pub(crate) finished_at: Option<String>,
    pub(crate) duration_ms: f64,
    pub(crate) span_count: u32,
    pub(crate) error_count: u32,
    pub(crate) file_count: u32,
    pub(crate) page_count: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticSpanRow {
    pub(crate) span_id: String,
    pub(crate) trace_id: String,
    pub(crate) parent_span_id: Option<String>,
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) name: String,
    pub(crate) pipeline_step: String,
    pub(crate) category: String,
    pub(crate) annotation_engine: Option<String>,
    pub(crate) status: String,
    pub(crate) started_at: String,
    pub(crate) ended_at: String,
    pub(crate) duration_ms: f64,
    pub(crate) attributes: Value,
    pub(crate) error_type: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) error_stack: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticEventRow {
    pub(crate) event_id: String,
    pub(crate) trace_id: String,
    pub(crate) span_id: Option<String>,
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) timestamp: String,
    pub(crate) event_type: String,
    pub(crate) name: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) attributes: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticWorkUnitRow {
    pub(crate) work_unit_id: String,
    pub(crate) run_id: String,
    pub(crate) work_key: String,
    pub(crate) file_hash: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) source_path: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) phase: String,
    pub(crate) engine: String,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) profile: Option<String>,
    pub(crate) execution_key: String,
    pub(crate) artifact_variant: Option<String>,
    pub(crate) status: String,
    pub(crate) attempt_count: u32,
    pub(crate) started_at: Option<String>,
    pub(crate) finished_at: Option<String>,
    pub(crate) duration_ms: Option<f64>,
    pub(crate) error: Option<String>,
    pub(crate) result: Value,
    pub(crate) metadata: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticModelLeaseRow {
    pub(crate) lease_id: String,
    pub(crate) run_id: String,
    pub(crate) execution_key: String,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) requested_context_tokens: Option<u32>,
    pub(crate) verified_context_tokens: Option<u32>,
    pub(crate) status: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: Option<String>,
    pub(crate) duration_ms: Option<f64>,
    pub(crate) error: Option<String>,
    pub(crate) metadata: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkUnitUpsert {
    pub(crate) work_unit_id: String,
    pub(crate) run_id: String,
    pub(crate) work_key: String,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) phase: String,
    pub(crate) engine: String,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) profile: Option<String>,
    pub(crate) execution_key: String,
    pub(crate) artifact_variant: Option<String>,
    pub(crate) metadata: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct DownloadEventInsert {
    pub(crate) event_id: String,
    pub(crate) download_id: String,
    pub(crate) download_key: String,
    pub(crate) owner_kind: String,
    pub(crate) owner_id: String,
    pub(crate) file_id: String,
    pub(crate) file_name: String,
    pub(crate) target_path: String,
    pub(crate) source_url: String,
    pub(crate) event_type: String,
    pub(crate) status: String,
    pub(crate) downloaded_bytes: u64,
    pub(crate) total_bytes: Option<u64>,
    pub(crate) error: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticSpanInsert {
    pub(crate) span_id: String,
    pub(crate) trace_id: String,
    pub(crate) parent_span_id: Option<String>,
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) name: String,
    pub(crate) pipeline_step: String,
    pub(crate) category: String,
    pub(crate) annotation_engine: Option<String>,
    pub(crate) status: String,
    pub(crate) started_at: String,
    pub(crate) ended_at: String,
    pub(crate) duration_ms: f64,
    pub(crate) attributes: Value,
    pub(crate) error_type: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) error_stack: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticEventInsert {
    pub(crate) event_id: String,
    pub(crate) trace_id: String,
    pub(crate) span_id: Option<String>,
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) timestamp: String,
    pub(crate) event_type: String,
    pub(crate) name: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) attributes: Value,
}

#[derive(Debug, Default)]
pub(crate) struct StoredSnapshot {
    pub(crate) runs: Vec<StoredRun>,
    pub(crate) completion_manifests: Vec<StoredRunCompletionManifest>,
    pub(crate) run_documents: Vec<StoredRunDocument>,
    pub(crate) documents: Vec<StoredDocument>,
    pub(crate) pages: Vec<StoredPage>,
}
