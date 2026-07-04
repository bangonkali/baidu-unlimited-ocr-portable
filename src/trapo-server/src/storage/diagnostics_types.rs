#[derive(Debug, Clone)]
pub(crate) struct DiagnosticModelLeaseInsert {
    pub(crate) run_id: String,
    pub(crate) execution_key: String,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) status: String,
    pub(crate) metadata: Value,
}
