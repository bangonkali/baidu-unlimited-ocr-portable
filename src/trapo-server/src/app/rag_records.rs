fn pipeline_task_record(row: PipelineTaskRow) -> PipelineTaskRecord {
    PipelineTaskRecord {
        task_id: row.task_id,
        task_kind: row.task_kind,
        origin_run_id: row.origin_run_id,
        status: row.status,
        params: row.params,
        result: row.result,
        queued_at: row.queued_at,
        started_at: row.started_at,
        finished_at: row.finished_at,
        runner_id: row.runner_id,
        error: row.error,
    }
}

fn grouped_search_hits(hits: Vec<RagSearchHitRow>) -> Vec<HybridSearchFileResult> {
    let mut files = BTreeMap::<String, Vec<HybridSearchHit>>::new();
    for hit in hits {
        files
            .entry(hit.file_hash.clone())
            .or_default()
            .push(HybridSearchHit {
                segment_id: hit.segment_id,
                file_hash: hit.file_hash,
                page_no: hit.page_no,
                annotation_id: hit.annotation_id,
                category: hit.category,
                text: hit.text,
                score: hit.score,
                hit_source: hit.hit_source,
                model_id: hit.model_id,
            });
    }
    files
        .into_iter()
        .map(|(file_hash, hits)| HybridSearchFileResult {
            file_hash,
            hit_count: usize_to_u32_saturating(hits.len()),
            hits,
        })
        .collect()
}

fn append_page_segments(
    source_run_id: &str,
    file_hash: &str,
    pages: Vec<PageTextRecord>,
    segments: &mut Vec<RagTextSegmentRow>,
) {
    for page in pages {
        let text = page.text.trim();
        if text.is_empty() {
            continue;
        }
        let segment_index = usize_to_u32_saturating(segments.len());
        segments.push(RagTextSegmentRow {
            segment_id: new_persistence_id(),
            source_run_id: source_run_id.to_string(),
            file_hash: file_hash.to_string(), // skylos: ignore[SKY-D215] file_hash is a digest key, not a filesystem path.
            page_no: page.page_no,
            segment_index,
            annotation_id: None,
            category: "page_text".to_string(),
            text: text.to_string(),
            token_estimate: estimate_tokens(text),
            text_start: 0,
            text_end: usize_to_u64_saturating(text.len()),
            source_kind: "page".to_string(),
        });
    }
}

fn estimate_tokens(text: &str) -> u32 {
    let words = text.split_whitespace().count();
    usize_to_u32_saturating(words.saturating_mul(4).saturating_add(2) / 3)
}
