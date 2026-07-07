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
