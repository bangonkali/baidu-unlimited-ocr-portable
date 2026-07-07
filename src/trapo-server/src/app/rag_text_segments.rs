fn append_page_segments(
    source_run_id: &str,
    file_hash: &str,
    pages: Vec<PageTextRecord>,
    segments: &mut Vec<RagTextSegmentRow>,
) {
    for page in pages {
        for source in rag_text_sources(&page.text) {
            for (start, end) in rag_text_chunks(&source.text) {
                let slice = &source.text[start..end];
                let chunk = slice.trim();
                if chunk.is_empty() {
                    continue;
                }
                let leading_trim = slice.len() - slice.trim_start().len();
                let trailing_trim = slice.trim_end().len();
                let text_start = source.offset.saturating_add(start).saturating_add(leading_trim);
                let text_end = source.offset.saturating_add(start).saturating_add(trailing_trim);
                let segment_index = usize_to_u32_saturating(segments.len());
                let annotation_id = annotation_id_for_segment(&page, text_start, text_end);
                segments.push(RagTextSegmentRow { // skylos: ignore[SKY-D215] file_hash is a digest key, not a filesystem path.
                    segment_id: new_persistence_id(),
                    source_run_id: source_run_id.to_string(),
                    file_hash: file_hash.to_string(),
                    page_no: page.page_no,
                    segment_index,
                    annotation_id,
                    category: source.category.clone(),
                    text: chunk.to_string(),
                    token_estimate: estimate_tokens(chunk),
                    text_start: usize_to_u64_saturating(text_start),
                    text_end: usize_to_u64_saturating(text_end),
                    source_kind: "page_chunk".to_string(),
                });
            }
        }
    }
}

fn annotation_id_for_segment(
    page: &PageTextRecord,
    text_start: usize,
    text_end: usize,
) -> Option<String> {
    let start = usize_to_u64_saturating(text_start);
    let end = usize_to_u64_saturating(text_end);
    page.spans
        .iter()
        .find(|span| {
            !span.annotation_id.is_empty()
                && span.page_no == page.page_no
                && ranges_overlap(start, end, span.start, span.end)
        })
        .map(|span| span.annotation_id.clone())
}

const fn ranges_overlap(left_start: u64, left_end: u64, right_start: u64, right_end: u64) -> bool {
    left_start < right_end && right_start < left_end
}

fn rag_text_segments_are_current(segments: &[RagTextSegmentRow]) -> bool {
    !segments.is_empty()
        && segments.iter().all(|segment| {
            segment.source_kind == "page_chunk"
                && segment.token_estimate <= RAG_SEGMENT_TARGET_TOKENS
                && rag_segment_text_is_clean(&segment.text)
        })
}

