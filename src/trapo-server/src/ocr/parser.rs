pub fn parse_ocr_markers(raw_text: &str, context: &ParseContext) -> ParsedOcrPage {
    let segments = collect_segments(raw_text);
    let mut parsed = ParsedOcrPage {
        raw_text: raw_text.to_string(),
        ..ParsedOcrPage::default()
    };
    let mut cursor = 0;
    for segment in &segments {
        if segment.start > cursor {
            parsed
                .cleaned_text
                .push_str(&remove_marker_tokens(&raw_text[cursor..segment.start]));
        }
        let clean_start = parsed.cleaned_text.len() as u64;
        parsed.cleaned_text.push_str(&segment.label);
        for box_points in &segment.boxes {
            let region_id = region_id_for(context, segment, box_points);
            parsed.spans.push(TextRegionSpan {
                region_id: region_id.clone(),
                page_no: context.page_no,
                start: clean_start,
                end: clean_start,
            });
            parsed.boxes.push(OverlayBox {
                region_id,
                label: segment.label.clone(),
                content_markdown: segment.label.clone(),
                content_html: None,
                page_no: context.page_no,
                left_percent: box_points.x1 / 999.0 * 100.0,
                top_percent: box_points.y1 / 999.0 * 100.0,
                width_percent: (box_points.x2 - box_points.x1) / 999.0 * 100.0,
                height_percent: (box_points.y2 - box_points.y1) / 999.0 * 100.0,
                hidden: false,
            });
        }
        cursor = cursor.max(segment.end);
    }
    if cursor < raw_text.len() {
        parsed
            .cleaned_text
            .push_str(&remove_marker_tokens(&raw_text[cursor..]));
    }
    parsed
}

pub fn apply_region_content(parsed: &mut ParsedOcrPage) {
    let scope_boundaries = sorted_unique_span_starts(&parsed.spans);
    for item in &mut parsed.boxes {
        if let Some(span) = parsed
            .spans
            .iter()
            .find(|span| span.region_id == item.region_id)
        {
            let start = span.start as usize;
            let end = next_scope_end(span.start, &scope_boundaries, parsed.cleaned_text.len());
            if start <= end && end <= parsed.cleaned_text.len() {
                item.content_markdown = parsed.cleaned_text[start..end].trim().to_string();
            }
        }
    }
}

fn sorted_unique_span_starts(spans: &[TextRegionSpan]) -> Vec<u64> {
    let mut starts = spans.iter().map(|span| span.start).collect::<Vec<_>>();
    starts.sort_unstable();
    starts.dedup();
    starts
}

fn next_scope_end(current_start: u64, boundaries: &[u64], text_len: usize) -> usize {
    boundaries
        .iter()
        .copied()
        .find(|start| *start > current_start)
        .map_or(text_len, |start| start as usize)
}
