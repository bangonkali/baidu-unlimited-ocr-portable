use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct RealtimeEventRecord {
    pub(crate) event_id: String,
    pub(crate) sequence: u64,
    #[serde(rename = "type")]
    pub(crate) event_type: String,
    pub(crate) occurred_at: String,
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    #[schema(value_type = Object)]
    pub(crate) payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct OcrReplayPayload {
    pub(crate) events: Vec<RealtimeEventRecord>,
    pub(crate) next_since_sequence: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticRunRecord {
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticRunsPayload {
    pub(crate) runs: Vec<DiagnosticRunRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticSpanRecord {
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
    #[schema(value_type = Object)]
    pub(crate) attributes: Value,
    pub(crate) error_type: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) error_stack: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticEventRecord {
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
    #[schema(value_type = Object)]
    pub(crate) attributes: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticTraceSummary {
    pub(crate) run_id: Option<String>,
    pub(crate) span_count: u32,
    pub(crate) event_count: u32,
    pub(crate) error_count: u32,
    pub(crate) total_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticTracePayload {
    pub(crate) summary: DiagnosticTraceSummary,
    pub(crate) spans: Vec<DiagnosticSpanRecord>,
    pub(crate) events: Vec<DiagnosticEventRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticWorkUnitRecord {
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
    #[schema(value_type = Object)]
    pub(crate) result: Value,
    #[schema(value_type = Object)]
    pub(crate) metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticModelLeaseRecord {
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
    #[schema(value_type = Object)]
    pub(crate) metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticProgressSummary {
    pub(crate) total_work_units: u32,
    pub(crate) queued: u32,
    pub(crate) running: u32,
    pub(crate) completed: u32,
    pub(crate) failed: u32,
    pub(crate) cancelled: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticProgressPayload {
    pub(crate) summary: DiagnosticProgressSummary,
    pub(crate) work_units: Vec<DiagnosticWorkUnitRecord>,
    pub(crate) model_leases: Vec<DiagnosticModelLeaseRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticBreakdownRecord {
    pub(crate) key: String,
    pub(crate) count: u32,
    pub(crate) total_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticSlowSpanRecord {
    pub(crate) span_id: String,
    pub(crate) name: String,
    pub(crate) pipeline_step: String,
    pub(crate) duration_ms: f64,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticRecommendationRecord {
    pub(crate) severity: String,
    pub(crate) title: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticAnalyticsSummary {
    pub(crate) span_count: u32,
    pub(crate) event_count: u32,
    pub(crate) error_count: u32,
    pub(crate) total_duration_ms: f64,
    pub(crate) average_span_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticAnalyticsPayload {
    pub(crate) summary: DiagnosticAnalyticsSummary,
    pub(crate) by_pipeline_step: Vec<DiagnosticBreakdownRecord>,
    pub(crate) by_category: Vec<DiagnosticBreakdownRecord>,
    pub(crate) slow_spans: Vec<DiagnosticSlowSpanRecord>,
    pub(crate) recommendations: Vec<DiagnosticRecommendationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DiagnosticModelsPayload {
    pub(crate) model_leases: Vec<DiagnosticModelLeaseRecord>,
}
