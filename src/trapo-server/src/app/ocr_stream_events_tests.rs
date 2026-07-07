use super::*;

#[test]
fn token_events_match_client_stream_contract() -> anyhow::Result<()> {
    let hub = RealtimeHub::new();
    let mut receiver = hub.subscribe();
    let mut telemetry = OcrStreamTelemetry::new();

    publish_token_events(&hub, None, &stream_context(), &mut telemetry, "Invoice", 0);

    let patch = receiver.try_recv()?;
    assert_eq!(patch.event_type, "ocr.page.text.patch");
    assert_eq!(patch.payload["run_id"], "run-a");
    assert_eq!(patch.payload["file_hash"], "file-a");
    assert_eq!(patch.payload["page_no"], 1);
    assert_eq!(patch.payload["op"], "append");
    assert_eq!(patch.payload["start"], 0);
    assert_eq!(patch.payload["end"], 0);
    assert_eq!(patch.payload["text"], "Invoice");
    assert!(receiver.try_recv().is_err());
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
        None,
        &context,
        &mut telemetry,
        "A <|ref|>Total<|/ref|>",
        0,
    );
    let patch = receiver.try_recv()?;
    assert_eq!(patch.event_type, "ocr.page.text.patch");
    assert_eq!(patch.payload["op"], "replace");
    assert_eq!(patch.payload["text"], "A ");
    assert!(receiver.try_recv().is_err());

    publish_token_events(
        &hub,
        None,
        &context,
        &mut telemetry,
        "<|det|>[[0,0,999,100]]<|/det|>",
        1,
    );
    let span = receiver.try_recv()?;
    assert_eq!(span.event_type, "ocr.page.span.upsert");
    assert_eq!(span.payload["span"]["start"], 2);
    assert_eq!(span.payload["span"]["end"], 2);
    let region = receiver.try_recv()?;
    assert_eq!(region.event_type, "ocr.page.region.upsert");
    assert_eq!(region.payload["region"]["label"], "Total");
    assert_eq!(region.payload["region"]["page_no"], 1);

    publish_token_events(&hub, None, &context, &mut telemetry, " tail", 2);
    assert!(receiver.try_recv().is_err());
    finish_token_events(&hub, &context, &mut telemetry);
    let patch = receiver.try_recv()?;
    assert_eq!(patch.event_type, "ocr.page.text.patch");
    assert_eq!(patch.payload["op"], "replace");
    assert_eq!(patch.payload["text"], "A tail");
    assert!(receiver.try_recv().is_err());
    Ok(())
}

#[test]
fn streamed_regions_get_uuid_v7_ids_without_database_write() -> anyhow::Result<()> {
    let hub = RealtimeHub::new();
    let identities = AnnotationIdentityRuntime::new_for_tests();
    let mut receiver = hub.subscribe();
    let mut telemetry = OcrStreamTelemetry::new();
    let context = stream_context();

    publish_token_events(
        &hub,
        Some(&identities),
        &context,
        &mut telemetry,
        "A <|ref|>Total<|/ref|><|det|>[[0,0,999,100]]<|/det|>",
        0,
    );

    assert_eq!(receiver.try_recv()?.event_type, "ocr.page.text.patch");
    let span = receiver.try_recv()?;
    let region = receiver.try_recv()?;
    let span_id = span.payload["span"]["annotation_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("span annotation_id missing"))?;
    let region_id = region.payload["region"]["annotation_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("region annotation_id missing"))?;
    assert_eq!(span.event_type, "ocr.page.span.upsert");
    assert_eq!(region.event_type, "ocr.page.region.upsert");
    assert_eq!(span_id, region_id);
    assert!(crate::ids::is_uuid_v7(span_id));

    let pending_len = {
        let pending = identities
            .pending
            .lock()
            .map_err(|_| anyhow::anyhow!("pending annotations mutex poisoned"))?;
        pending.len()
    };
    assert_eq!(pending_len, 1);
    Ok(())
}

fn stream_context() -> OcrStreamContext {
    OcrStreamContext {
        run_id: "run-a".to_string(),
        run_engine_id: "01980a3d-a4fc-7000-8000-000000000001".to_string(),
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
