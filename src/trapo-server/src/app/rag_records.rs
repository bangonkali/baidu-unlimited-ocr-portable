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
