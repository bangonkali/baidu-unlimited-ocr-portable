const fn engine_completion_status(output_count: u32, failed: bool, fallback: bool) -> &'static str {
    if output_count == 0 && failed {
        "failed"
    } else if failed || fallback {
        "completed_with_errors"
    } else {
        "completed"
    }
}

#[derive(Default)]
struct EngineRunOutcome {
    output_count: u32,
    failed: bool,
}

struct IngestEngineSpanFinish<'a> {
    config: &'a RunEngineConfigState,
    fallback_reason: Option<&'a str>,
    model_id: &'a str,
    output_count: u32,
    profile_id: &'a str,
    run_id: &'a str,
    runtime_id: &'a str,
    status: &'a str,
}

struct IngestExecution {
    completed_pages: BTreeSet<(String, u32)>,
    embedding_after_ingest: bool,
    embedding_dimension: Option<u32>,
    embedding_model_id: Option<String>,
    engine_configs: Vec<RunEngineConfigState>,
    files: Vec<DiscoveredFile>,
    run_id: String,
    text_index_after_ingest: bool,
}
