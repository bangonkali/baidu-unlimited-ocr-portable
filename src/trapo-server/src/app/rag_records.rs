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
        for (start, end) in rag_text_chunks(text) {
            let chunk = text[start..end].trim();
            if chunk.is_empty() {
                continue;
            }
            let leading_trim = text[start..end].len() - text[start..end].trim_start().len();
            let trailing_trim = text[start..end].trim_end().len();
            let text_start = start.saturating_add(leading_trim);
            let text_end = start.saturating_add(trailing_trim);
            let segment_index = usize_to_u32_saturating(segments.len());
            segments.push(RagTextSegmentRow {
                segment_id: new_persistence_id(),
                source_run_id: source_run_id.to_string(),
                file_hash: file_hash.to_string(), // skylos: ignore[SKY-D215] file_hash is a digest key, not a filesystem path.
                page_no: page.page_no,
                segment_index,
                annotation_id: None,
                category: "page_text".to_string(),
                text: chunk.to_string(),
                token_estimate: estimate_tokens(chunk),
                text_start: usize_to_u64_saturating(text_start),
                text_end: usize_to_u64_saturating(text_end),
                source_kind: "page_chunk".to_string(),
            });
        }
    }
}

const RAG_SEGMENT_TARGET_TOKENS: u32 = 192;
const RAG_SEGMENT_OVERLAP_TOKENS: u32 = 32;
const TOKEN_UNITS_PER_TOKEN: u32 = 4;

fn rag_text_segments_are_current(segments: &[RagTextSegmentRow]) -> bool {
    !segments.is_empty()
        && segments.iter().all(|segment| {
            segment.source_kind == "page_chunk"
                && segment.token_estimate <= RAG_SEGMENT_TARGET_TOKENS
        })
}

fn rag_text_chunks(text: &str) -> Vec<(usize, usize)> {
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let end = chunk_end(text, start);
        if end <= start {
            break;
        }
        chunks.push((start, end));
        if end == text.len() {
            break;
        }
        let overlapped = overlap_start(text, start, end);
        start = if overlapped <= start { end } else { overlapped };
    }
    chunks
}

fn chunk_end(text: &str, start: usize) -> usize {
    let target_units = RAG_SEGMENT_TARGET_TOKENS.saturating_mul(TOKEN_UNITS_PER_TOKEN);
    let minimum_boundary_units = target_units / 2;
    let mut units = 0_u32;
    let mut last_boundary = None;
    for (offset, character) in text[start..].char_indices() {
        let next = start + offset + character.len_utf8();
        units = units.saturating_add(char_token_units(character));
        if units >= minimum_boundary_units && is_chunk_boundary(character) {
            last_boundary = Some(next);
        }
        if units >= target_units {
            return last_boundary.filter(|boundary| *boundary > start).unwrap_or(next);
        }
    }
    text.len()
}

fn overlap_start(text: &str, start: usize, end: usize) -> usize {
    let target_units = RAG_SEGMENT_OVERLAP_TOKENS.saturating_mul(TOKEN_UNITS_PER_TOKEN);
    let mut units = 0_u32;
    let mut candidate = end;
    for (offset, character) in text[..end].char_indices().rev() {
        units = units.saturating_add(char_token_units(character));
        candidate = offset;
        if units >= target_units {
            break;
        }
    }
    if candidate <= start { end } else { candidate }
}

fn estimate_tokens(text: &str) -> u32 {
    let units = text
        .chars()
        .map(char_token_units)
        .fold(0_u32, u32::saturating_add);
    units
        .saturating_add(TOKEN_UNITS_PER_TOKEN - 1)
        .saturating_div(TOKEN_UNITS_PER_TOKEN)
}

fn char_token_units(character: char) -> u32 {
    if character.is_whitespace() {
        0
    } else if is_cjk(character) {
        TOKEN_UNITS_PER_TOKEN
    } else {
        1
    }
}

fn is_cjk(character: char) -> bool {
    matches!(
        u32::from(character),
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x3040..=0x30FF
            | 0xAC00..=0xD7AF
    )
}

const fn is_chunk_boundary(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | ')'
                | ']'
                | '}'
                | '\u{3002}'
                | '\u{ff0c}'
                | '\u{ff1b}'
                | '\u{ff1a}'
                | '\u{ff01}'
                | '\u{ff1f}'
        )
}

#[cfg(test)]
mod rag_chunk_tests {
    use super::*;

    #[test]
    fn cjk_text_is_chunked_under_embedding_batch_limit() {
        let text = "饮用水卫生标准".repeat(180);
        let chunks = rag_text_chunks(&text);

        assert!(chunks.len() > 1);
        for (start, end) in chunks {
            let chunk = &text[start..end];
            assert!(estimate_tokens(chunk) <= RAG_SEGMENT_TARGET_TOKENS);
        }
    }

    #[test]
    fn append_page_segments_preserves_offsets_for_chunks() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: "水".repeat(900),
                spans: Vec::new(),
            }],
            &mut segments,
        );

        assert!(segments.len() > 1);
        assert_eq!(segments[0].source_kind, "page_chunk");
        assert_eq!(segments[0].text_start, 0);
        assert!(segments[0].text_end > segments[0].text_start);
        assert!(segments
            .iter()
            .all(|segment| segment.token_estimate <= RAG_SEGMENT_TARGET_TOKENS));
    }

    #[test]
    fn stale_or_oversized_text_segments_are_rebuilt() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: "short text".to_string(),
                spans: Vec::new(),
            }],
            &mut segments,
        );

        assert!(rag_text_segments_are_current(&segments));
        segments[0].source_kind = "page".to_string();
        assert!(!rag_text_segments_are_current(&segments));
        segments[0].source_kind = "page_chunk".to_string();
        segments[0].token_estimate = RAG_SEGMENT_TARGET_TOKENS + 1;
        assert!(!rag_text_segments_are_current(&segments));
    }
}
