#[derive(Debug, Clone)]
struct OcrStreamContext {
    run_id: String,
    run_engine_id: String,
    file_hash: String,
    page_no: u32,
    engine_id: String,
    profile_id: String,
    model_id: String,
    runtime_id: String,
    runtime_platform: String,
    accelerator: String,
}

const TEXT_PATCH_FLUSH_BYTES: usize = 4096;
const TEXT_PATCH_FLUSH_MS: u64 = 150;

struct OcrStreamTelemetry {
    raw_text: String,
    raw_end: usize,
    emitted_region_ids: std::collections::HashSet<String>,
    last_text_patch_at: Instant,
    last_text_patch_len: usize,
    pending_append: String,
    pending_append_start: usize,
    pending_replace: Option<String>,
}

impl OcrStreamTelemetry {
    fn new() -> Self {
        let started = Instant::now();
        Self {
            raw_text: String::new(),
            raw_end: 0,
            emitted_region_ids: std::collections::HashSet::new(),
            last_text_patch_at: started
                .checked_sub(std::time::Duration::from_millis(TEXT_PATCH_FLUSH_MS))
                .unwrap_or(started),
            last_text_patch_len: 0,
            pending_append: String::new(),
            pending_append_start: 0,
            pending_replace: None,
        }
    }

    fn record(&mut self, text: &str) -> OcrTokenTelemetry {
        let raw_start = self.raw_end;
        self.raw_text.push_str(text);
        self.raw_end = self.raw_end.saturating_add(text.len());
        OcrTokenTelemetry { raw_start }
    }

    fn queue_text_patch(&mut self, token_text: &str, token_raw_start: usize, cleaned_text: String) {
        if cleaned_text != self.raw_text {
            self.pending_append.clear();
            self.pending_replace = Some(cleaned_text);
            return;
        }
        if self.pending_replace.is_some() {
            self.pending_replace = Some(cleaned_text);
            return;
        }
        if self.pending_append.is_empty() {
            self.pending_append_start = token_raw_start;
        }
        self.pending_append.push_str(token_text);
    }

    fn should_flush_text_patch(&self) -> bool {
        self.pending_text_bytes() >= TEXT_PATCH_FLUSH_BYTES
            || self.last_text_patch_at.elapsed()
                >= std::time::Duration::from_millis(TEXT_PATCH_FLUSH_MS)
    }

    fn flush_text_patch(&mut self, hub: &RealtimeHub, context: &OcrStreamContext) {
        if let Some(text) = self.pending_replace.take() {
            let text_len = text.len();
            let mut patch = stream_context_payload(context);
            patch["op"] = json!("replace");
            patch["start"] = json!(0);
            patch["end"] = json!(self.last_text_patch_len);
            patch["text"] = json!(text);
            self.last_text_patch_len = text_len;
            self.last_text_patch_at = Instant::now();
            hub.publish("ocr.page.text.patch", patch);
            return;
        }
        if self.pending_append.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.pending_append);
        let text_len = text.len();
        let mut patch = stream_context_payload(context);
        patch["op"] = json!("append");
        patch["start"] = json!(self.pending_append_start);
        patch["end"] = json!(self.pending_append_start);
        patch["text"] = json!(text);
        self.last_text_patch_len = self.last_text_patch_len.saturating_add(text_len);
        self.last_text_patch_at = Instant::now();
        hub.publish("ocr.page.text.patch", patch);
    }

    fn pending_text_bytes(&self) -> usize {
        self.pending_replace
            .as_ref()
            .map_or_else(|| self.pending_append.len(), String::len)
    }
}

struct OcrTokenTelemetry {
    raw_start: usize,
}

fn publish_token_events(
    hub: &RealtimeHub,
    annotation_identities: Option<&AnnotationIdentityRuntime>,
    context: &OcrStreamContext,
    telemetry: &mut OcrStreamTelemetry,
    text: &str,
    _index: u64,
) {
    let token = telemetry.record(text);
    let parsed = crate::ocr::parse_ocr_markers(&telemetry.raw_text, &stream_parse_context(context));
    let cleaned_text = if parsed.cleaned_text.is_empty() {
        telemetry.raw_text.clone()
    } else {
        parsed.cleaned_text
    };
    telemetry.queue_text_patch(text, token.raw_start, cleaned_text);
    if telemetry.should_flush_text_patch() {
        telemetry.flush_text_patch(hub, context);
    }

    publish_region_events(hub, annotation_identities, context, telemetry);
}

fn finish_token_events(
    hub: &RealtimeHub,
    context: &OcrStreamContext,
    telemetry: &mut OcrStreamTelemetry,
) {
    telemetry.flush_text_patch(hub, context);
}

fn publish_region_events(
    hub: &RealtimeHub,
    annotation_identities: Option<&AnnotationIdentityRuntime>,
    context: &OcrStreamContext,
    telemetry: &mut OcrStreamTelemetry,
) {
    let mut parsed = crate::ocr::parse_ocr_markers(&telemetry.raw_text, &stream_parse_context(context));
    crate::ocr::apply_region_content(&mut parsed);
    if let Some(annotation_identities) = annotation_identities {
        apply_stream_annotation_identities(annotation_identities, context, &mut parsed);
    }
    let spans_by_region = parsed
        .spans
        .iter()
        .map(|span| (span.region_id.clone(), span.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    for region in parsed.boxes {
        if !telemetry
            .emitted_region_ids
            .insert(region.region_id.clone())
        {
            continue;
        }
        if let Some(span) = spans_by_region.get(&region.region_id) {
            let mut payload = stream_context_payload(context);
            payload["span"] = json!(span);
            hub.publish("ocr.page.span.upsert", payload);
        }
        let mut payload = stream_context_payload(context);
        payload["region"] = json!(region);
        hub.publish("ocr.page.region.upsert", payload);
    }
}

fn apply_stream_annotation_identities(
    annotation_identities: &AnnotationIdentityRuntime,
    context: &OcrStreamContext,
    parsed: &mut crate::ocr::ParsedOcrPage,
) {
    let boxes = parsed.boxes.clone();
    for (index, box_record) in boxes.iter().enumerate() {
        let span = parsed
            .spans
            .iter()
            .find(|item| item.source_region_key == box_record.source_region_key);
        let draft = annotation_identity_draft(
            AnnotationDraftScope {
                run_id: &context.run_id,
                file_hash: &context.file_hash,
                engine_id: &context.engine_id,
                profile_id: &context.profile_id,
                index,
            },
            box_record,
            span,
        );
        let resolved = annotation_identities.resolve_and_enqueue(draft);
        apply_annotation_id(parsed, &box_record.source_region_key, &resolved.annotation_id);
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
        "run_engine_id": context.run_engine_id,
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

#[cfg(test)]
#[path = "ocr_stream_events_tests.rs"]
mod ocr_stream_events_tests;
