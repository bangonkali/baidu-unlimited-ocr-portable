use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RealtimeEventRecord {
    pub sequence: u64,
    #[serde(rename = "type")]
    pub event_type: String,
    pub occurred_at: String,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    #[schema(value_type = Object)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OcrReplayPayload {
    pub events: Vec<RealtimeEventRecord>,
    pub next_since_sequence: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticRunRecord {
    pub run_id: String,
    pub root_path: String,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: f64,
    pub span_count: u32,
    pub error_count: u32,
    pub file_count: u32,
    pub page_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticRunsPayload {
    pub runs: Vec<DiagnosticRunRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticSpanRecord {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub name: String,
    pub pipeline_step: String,
    pub category: String,
    pub annotation_engine: Option<String>,
    pub status: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: f64,
    #[schema(value_type = Object)]
    pub attributes: Value,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_stack: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticEventRecord {
    pub event_id: String,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub timestamp: String,
    pub event_type: String,
    pub name: String,
    pub severity: String,
    pub message: String,
    #[schema(value_type = Object)]
    pub attributes: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticTraceSummary {
    pub run_id: Option<String>,
    pub span_count: u32,
    pub event_count: u32,
    pub error_count: u32,
    pub total_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticTracePayload {
    pub summary: DiagnosticTraceSummary,
    pub spans: Vec<DiagnosticSpanRecord>,
    pub events: Vec<DiagnosticEventRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticWorkUnitRecord {
    pub work_unit_id: String,
    pub run_id: String,
    pub work_key: String,
    pub file_hash: Option<String>,
    pub filename: Option<String>,
    pub source_path: Option<String>,
    pub page_no: Option<u32>,
    pub phase: String,
    pub engine: String,
    pub provider: String,
    pub model: String,
    pub profile: Option<String>,
    pub execution_key: String,
    pub artifact_variant: Option<String>,
    pub status: String,
    pub attempt_count: u32,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: Option<f64>,
    pub error: Option<String>,
    #[schema(value_type = Object)]
    pub result: Value,
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticModelLeaseRecord {
    pub lease_id: String,
    pub run_id: String,
    pub execution_key: String,
    pub provider: String,
    pub model: String,
    pub requested_context_tokens: Option<u32>,
    pub verified_context_tokens: Option<u32>,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<f64>,
    pub error: Option<String>,
    #[schema(value_type = Object)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticProgressSummary {
    pub total_work_units: u32,
    pub queued: u32,
    pub running: u32,
    pub completed: u32,
    pub failed: u32,
    pub cancelled: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticProgressPayload {
    pub summary: DiagnosticProgressSummary,
    pub work_units: Vec<DiagnosticWorkUnitRecord>,
    pub model_leases: Vec<DiagnosticModelLeaseRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticBreakdownRecord {
    pub key: String,
    pub count: u32,
    pub total_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticSlowSpanRecord {
    pub span_id: String,
    pub name: String,
    pub pipeline_step: String,
    pub duration_ms: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticRecommendationRecord {
    pub severity: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticAnalyticsSummary {
    pub span_count: u32,
    pub event_count: u32,
    pub error_count: u32,
    pub total_duration_ms: f64,
    pub average_span_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticAnalyticsPayload {
    pub summary: DiagnosticAnalyticsSummary,
    pub by_pipeline_step: Vec<DiagnosticBreakdownRecord>,
    pub by_category: Vec<DiagnosticBreakdownRecord>,
    pub slow_spans: Vec<DiagnosticSlowSpanRecord>,
    pub recommendations: Vec<DiagnosticRecommendationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiagnosticModelsPayload {
    pub model_leases: Vec<DiagnosticModelLeaseRecord>,
}
