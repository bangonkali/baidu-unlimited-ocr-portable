#[derive(Debug, Clone)]
struct OcrStreamContext {
    run_id: String,
    file_hash: String,
    page_no: u32,
    engine_id: String,
    profile_id: String,
    model_id: String,
    runtime_id: String,
    runtime_platform: String,
    accelerator: String,
}

struct OcrStreamTelemetry {
    started: Instant,
    raw_text: String,
    raw_end: usize,
    token_count: u64,
    emitted_region_ids: std::collections::HashSet<String>,
}

impl OcrStreamTelemetry {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            raw_text: String::new(),
            raw_end: 0,
            token_count: 0,
            emitted_region_ids: std::collections::HashSet::new(),
        }
    }

    fn record(&mut self, text: &str, index: u64) -> OcrTokenTelemetry {
        let raw_start = self.raw_end;
        self.raw_text.push_str(text);
        self.raw_end = self.raw_end.saturating_add(text.len());
        self.token_count = self.token_count.max(index.saturating_add(1));
        let elapsed_ms = self.started.elapsed().as_millis() as u64;
        OcrTokenTelemetry {
            raw_start,
            raw_end: self.raw_end,
            elapsed_ms,
            avg_tps: average_tps(self.token_count, elapsed_ms),
        }
    }
}

struct OcrTokenTelemetry {
    raw_start: usize,
    raw_end: usize,
    elapsed_ms: u64,
    avg_tps: f64,
}

fn publish_token_events(
    hub: &RealtimeHub,
    context: &OcrStreamContext,
    telemetry: &mut OcrStreamTelemetry,
    text: &str,
    index: u64,
) {
    let token = telemetry.record(text, index);
    let mut raw_delta = stream_context_payload(context);
    raw_delta["token_index"] = json!(index);
    raw_delta["delta"] = json!(text);
    raw_delta["raw_start"] = json!(token.raw_start);
    raw_delta["raw_end"] = json!(token.raw_end);
    raw_delta["elapsed_ms"] = json!(token.elapsed_ms);
    raw_delta["avg_tps"] = json!(token.avg_tps);
    hub.publish("ocr.page.raw.delta", raw_delta);

    let mut text_patch = stream_context_payload(context);
    text_patch["op"] = json!("append");
    text_patch["start"] = json!(token.raw_start);
    text_patch["end"] = json!(token.raw_start);
    text_patch["text"] = json!(text);
    hub.publish("ocr.page.text.patch", text_patch);

    publish_region_events(hub, context, telemetry);
}

fn publish_region_events(
    hub: &RealtimeHub,
    context: &OcrStreamContext,
    telemetry: &mut OcrStreamTelemetry,
) {
    let mut parsed = crate::ocr::parse_ocr_markers(&telemetry.raw_text, &stream_parse_context(context));
    crate::ocr::apply_region_content(&mut parsed);
    for region in parsed.boxes {
        if !telemetry
            .emitted_region_ids
            .insert(region.region_id.clone())
        {
            continue;
        }
        let mut payload = stream_context_payload(context);
        payload["region"] = json!(region);
        hub.publish("ocr.page.region.upsert", payload);
    }
}

fn stream_parse_context(context: &OcrStreamContext) -> crate::ocr::ParseContext {
    crate::ocr::ParseContext {
        file_hash: context.file_hash.clone(),
        page_no: context.page_no,
        engine_id: context.engine_id.clone(),
        profile_id: context.profile_id.clone(),
    }
}

fn stream_context_payload(context: &OcrStreamContext) -> serde_json::Value {
    json!({
        "run_id": context.run_id,
        "file_hash": context.file_hash,
        "page_no": context.page_no,
        "engine_id": context.engine_id,
        "profile_id": context.profile_id,
        "model_id": context.model_id,
        "runtime_id": context.runtime_id,
        "runtime_platform": context.runtime_platform,
        "accelerator": context.accelerator,
    })
}

fn stream_terminal_payload(
    context: &OcrStreamContext,
    status: &str,
    error: Option<&str>,
) -> serde_json::Value {
    let mut payload = stream_context_payload(context);
    payload["status"] = json!(status);
    payload["error"] = error.map_or(serde_json::Value::Null, serde_json::Value::from);
    payload
}

fn average_tps(token_count: u64, elapsed_ms: u64) -> f64 {
    if token_count == 0 || elapsed_ms == 0 {
        return 0.0;
    }
    token_count as f64 / (elapsed_ms as f64 / 1000.0)
}

#[cfg(test)]
mod ocr_stream_events_tests {
    use super::*;

    #[test]
    fn token_events_match_client_stream_contract() -> anyhow::Result<()> {
        let hub = RealtimeHub::new();
        let mut receiver = hub.subscribe();
        let mut telemetry = OcrStreamTelemetry::new();

        publish_token_events(&hub, &stream_context(), &mut telemetry, "Invoice", 0);

        let raw = receiver.try_recv()?;
        assert_eq!(raw.event_type, "ocr.page.raw.delta");
        assert_eq!(raw.payload["run_id"], "run-a");
        assert_eq!(raw.payload["file_hash"], "file-a");
        assert_eq!(raw.payload["page_no"], 1);
        assert_eq!(raw.payload["token_index"], 0);
        assert_eq!(raw.payload["delta"], "Invoice");
        assert_eq!(raw.payload["raw_start"], 0);
        assert_eq!(raw.payload["raw_end"], 7);

        let patch = receiver.try_recv()?;
        assert_eq!(patch.event_type, "ocr.page.text.patch");
        assert_eq!(patch.payload["op"], "append");
        assert_eq!(patch.payload["start"], 0);
        assert_eq!(patch.payload["end"], 0);
        assert_eq!(patch.payload["text"], "Invoice");
        Ok(())
    }

    #[test]
    fn region_event_is_emitted_when_box_marker_completes() -> anyhow::Result<()> {
        let hub = RealtimeHub::new();
        let mut receiver = hub.subscribe();
        let mut telemetry = OcrStreamTelemetry::new();
        let context = stream_context();

        publish_token_events(
            &hub,
            &context,
            &mut telemetry,
            "A <|ref|>Total<|/ref|>",
            0,
        );
        assert_eq!(receiver.try_recv()?.event_type, "ocr.page.raw.delta");
        assert_eq!(receiver.try_recv()?.event_type, "ocr.page.text.patch");
        assert!(receiver.try_recv().is_err());

        publish_token_events(
            &hub,
            &context,
            &mut telemetry,
            "<|det|>[[0,0,999,100]]<|/det|>",
            1,
        );
        assert_eq!(receiver.try_recv()?.event_type, "ocr.page.raw.delta");
        assert_eq!(receiver.try_recv()?.event_type, "ocr.page.text.patch");
        let region = receiver.try_recv()?;
        assert_eq!(region.event_type, "ocr.page.region.upsert");
        assert_eq!(region.payload["region"]["label"], "Total");
        assert_eq!(region.payload["region"]["page_no"], 1);

        publish_token_events(&hub, &context, &mut telemetry, " tail", 2);
        assert_eq!(receiver.try_recv()?.event_type, "ocr.page.raw.delta");
        assert_eq!(receiver.try_recv()?.event_type, "ocr.page.text.patch");
        assert!(receiver.try_recv().is_err());
        Ok(())
    }

    fn stream_context() -> OcrStreamContext {
        OcrStreamContext {
            run_id: "run-a".to_string(),
            file_hash: "file-a".to_string(),
            page_no: 1,
            engine_id: ENGINE_ID.to_string(),
            profile_id: "experimental-exact-prefill-q4".to_string(),
            model_id: "unlimited-ocr-q4-k-m".to_string(),
            runtime_id: "windows-x86_64-cuda13".to_string(),
            runtime_platform: "windows-x86_64-cuda13".to_string(),
            accelerator: "cuda".to_string(),
        }
    }
}
