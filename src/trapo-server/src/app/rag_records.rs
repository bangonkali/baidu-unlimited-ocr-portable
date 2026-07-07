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

const RAG_RRF_K: f64 = 60.0;

#[derive(Debug)]
struct RankedSearchResults {
    hits: Vec<HybridSearchHit>,
    files: Vec<HybridSearchFileResult>,
}

#[derive(Debug)]
struct SearchHitAccumulator {
    hit: RagSearchHitRow,
    relevance_score: f64,
    source_labels: Vec<String>,
}

impl SearchHitAccumulator {
    fn new(hit: RagSearchHitRow, relevance_score: f64) -> Self {
        Self {
            source_labels: vec![hit.hit_source.clone()],
            hit,
            relevance_score,
        }
    }

    fn add(&mut self, hit: RagSearchHitRow, relevance_score: f64) {
        self.relevance_score += relevance_score;
        if hit.score > self.hit.score {
            self.hit.score = hit.score;
        }
        if self.hit.model_id.is_none() {
            self.hit.model_id = hit.model_id;
        }
        self.add_source(hit.hit_source);
    }

    fn add_source(&mut self, source: String) {
        if !self.source_labels.iter().any(|label| label == &source) {
            self.source_labels.push(source);
        }
    }

    fn into_hit(self) -> HybridSearchHit {
        HybridSearchHit {
            segment_id: self.hit.segment_id,
            file_hash: self.hit.file_hash,
            page_no: self.hit.page_no,
            annotation_id: self.hit.annotation_id,
            category: self.hit.category,
            text: self.hit.text,
            score: self.hit.score,
            relevance_score: self.relevance_score,
            rank: 0,
            hit_source: combined_source_label(&self.source_labels),
            model_id: self.hit.model_id,
        }
    }
}

fn ranked_search_results(hits: Vec<RagSearchHitRow>, limit: u32) -> RankedSearchResults {
    let mut source_ranks = HashMap::<&'static str, usize>::new();
    let mut hits_by_segment = HashMap::<String, SearchHitAccumulator>::new();
    for hit in hits {
        let source_rank = next_source_rank(&mut source_ranks, source_family(&hit.hit_source));
        let relevance_score = reciprocal_rank_score(source_rank);
        let segment_id = hit.segment_id.clone();
        if let Some(accumulator) = hits_by_segment.get_mut(&segment_id) {
            accumulator.add(hit, relevance_score);
        } else {
            hits_by_segment.insert(segment_id, SearchHitAccumulator::new(hit, relevance_score));
        }
    }

    let mut ranked_hits = hits_by_segment
        .into_values()
        .map(SearchHitAccumulator::into_hit)
        .collect::<Vec<_>>();
    ranked_hits.sort_by(compare_search_hits);
    ranked_hits.truncate(usize::try_from(limit).unwrap_or(usize::MAX));
    for (index, hit) in ranked_hits.iter_mut().enumerate() {
        hit.rank = usize_to_u32_saturating(index.saturating_add(1));
    }

    RankedSearchResults {
        files: files_from_ranked_hits(&ranked_hits),
        hits: ranked_hits,
    }
}

fn next_source_rank(
    source_ranks: &mut HashMap<&'static str, usize>,
    source_family: &'static str,
) -> usize {
    let rank = source_ranks.entry(source_family).or_insert(0);
    *rank = rank.saturating_add(1);
    *rank
}

fn reciprocal_rank_score(source_rank: usize) -> f64 {
    1.0 / (RAG_RRF_K + f64::from(usize_to_u32_saturating(source_rank)))
}

fn files_from_ranked_hits(hits: &[HybridSearchHit]) -> Vec<HybridSearchFileResult> {
    let mut files_by_hash = HashMap::<String, HybridSearchFileResult>::new();
    for hit in hits {
        let entry =
            files_by_hash
                .entry(hit.file_hash.clone())
                .or_insert_with(|| HybridSearchFileResult {
                    file_hash: hit.file_hash.clone(),
                    hit_count: 0,
                    relevance_score: hit.relevance_score,
                    hits: Vec::new(),
                });
        entry.relevance_score = entry.relevance_score.max(hit.relevance_score);
        entry.hits.push(hit.clone());
    }

    let mut files = files_by_hash.into_values().collect::<Vec<_>>();
    for file in &mut files {
        file.hit_count = usize_to_u32_saturating(file.hits.len());
    }
    files.sort_by(compare_search_files);
    files
}

fn compare_search_hits(left: &HybridSearchHit, right: &HybridSearchHit) -> std::cmp::Ordering {
    right
        .relevance_score
        .total_cmp(&left.relevance_score)
        .then_with(|| left.file_hash.cmp(&right.file_hash))
        .then_with(|| left.page_no.cmp(&right.page_no))
        .then_with(|| left.segment_id.cmp(&right.segment_id))
}

fn compare_search_files(
    left: &HybridSearchFileResult,
    right: &HybridSearchFileResult,
) -> std::cmp::Ordering {
    right
        .relevance_score
        .total_cmp(&left.relevance_score)
        .then_with(|| left.file_hash.cmp(&right.file_hash))
}

fn combined_source_label(source_labels: &[String]) -> String {
    let mut labels = source_labels.to_vec();
    labels.sort_by(|left, right| {
        source_label_rank(left)
            .cmp(&source_label_rank(right))
            .then_with(|| left.cmp(right))
    });
    labels.join("+")
}

fn source_family(source: &str) -> &'static str {
    if source.starts_with("vss") {
        "vss"
    } else if source.starts_with("fts") {
        "fts"
    } else {
        "other"
    }
}

fn source_label_rank(source: &str) -> u8 {
    match source_family(source) {
        "fts" => 0,
        "vss" => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod rag_record_tests {
    use super::*;

    #[test]
    fn relevance_ranking_fuses_duplicate_sources_and_sorts_files() {
        let shared_segment_id = crate::ids::new_persistence_id();
        let second_segment_id = crate::ids::new_persistence_id();
        let results = ranked_search_results(
            vec![
                search_hit(&shared_segment_id, "file-z", 1, "fts", 12.0),
                search_hit(&second_segment_id, "file-a", 1, "fts", 11.0),
                search_hit(&shared_segment_id, "file-z", 1, "vss", 0.98),
            ],
            10,
        );

        assert_eq!(results.hits.len(), 2);
        assert_eq!(
            results.hits.first().map(|hit| hit.segment_id.as_str()),
            Some(shared_segment_id.as_str())
        );
        assert_eq!(
            results.hits.first().map(|hit| hit.hit_source.as_str()),
            Some("fts+vss")
        );
        assert_eq!(results.hits.first().map(|hit| hit.rank), Some(1));
        assert_eq!(results.hits.get(1).map(|hit| hit.rank), Some(2));
        assert!(
            results
                .hits
                .first()
                .zip(results.hits.get(1))
                .is_some_and(|(first, second)| first.relevance_score > second.relevance_score)
        );
        assert_eq!(
            results.files.first().map(|file| file.file_hash.as_str()),
            Some("file-z")
        );
    }

    #[test]
    fn relevance_ranking_applies_final_limit_after_fusion() {
        let shared_segment_id = crate::ids::new_persistence_id();
        let second_segment_id = crate::ids::new_persistence_id();
        let results = ranked_search_results(
            vec![
                search_hit(&shared_segment_id, "file-b", 1, "fts", 12.0),
                search_hit(&second_segment_id, "file-a", 1, "fts", 11.0),
                search_hit(&shared_segment_id, "file-b", 1, "vss", 0.98),
            ],
            1,
        );

        assert_eq!(results.hits.len(), 1);
        assert_eq!(results.files.len(), 1);
        assert_eq!(results.files.first().map(|file| file.hit_count), Some(1));
        assert_eq!(results.hits.first().map(|hit| hit.rank), Some(1));
    }

    fn search_hit(
        segment_id: &str,
        file_hash: &str,
        page_no: u32,
        hit_source: &str,
        score: f64,
    ) -> RagSearchHitRow {
        RagSearchHitRow {
            segment_id: segment_id.to_string(),
            file_hash: file_hash.to_string(),
            page_no,
            annotation_id: Some(crate::ids::new_persistence_id()),
            category: "page_text".to_string(),
            text: format!("{hit_source} search hit"),
            score,
            hit_source: hit_source.to_string(),
            model_id: hit_source
                .starts_with("vss")
                .then(|| "model-a".to_string()),
        }
    }
}
