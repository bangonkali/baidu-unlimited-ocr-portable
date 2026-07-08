#[must_use]
pub(crate) fn parse_ocr_markers(raw_text: &str, context: &ParseContext) -> ParsedOcrPage {
    let segments = collect_segments(raw_text);
    let mut parsed = ParsedOcrPage {
        raw_text: raw_text.to_string(),
        ..ParsedOcrPage::default()
    };
    let mut cursor = 0;
    for segment in &segments {
        if segment.start > cursor {
            push_cleaned_fragment(&mut parsed.cleaned_text, &raw_text[cursor..segment.start]);
        }
        let clean_start = usize_to_u64_saturating(parsed.cleaned_text.len());
        for (index, box_points) in segment.boxes.iter().enumerate() {
            let source_region_key = region_source_key_for(context, segment, box_points);
            let left_percent = box_points.x1 / 999.0 * 100.0;
            let top_percent = box_points.y1 / 999.0 * 100.0;
            let width_percent = (box_points.x2 - box_points.x1) / 999.0 * 100.0;
            let height_percent = (box_points.y2 - box_points.y1) / 999.0 * 100.0;
            let geometry = marker_geometry_for_box(
                segment,
                index,
                left_percent,
                top_percent,
                width_percent,
                height_percent,
            );
            parsed.spans.push(TextRegionSpan {
                annotation_id: source_region_key.clone(),
                region_id: source_region_key.clone(),
                source_region_key: source_region_key.clone(),
                page_no: context.page_no,
                start: clean_start,
                end: clean_start,
            });
            parsed.boxes.push(OverlayBox {
                annotation_id: source_region_key.clone(),
                region_id: source_region_key.clone(),
                source_region_key,
                label: segment.label.clone(),
                category: segment.label.clone(),
                content_markdown: String::new(),
                content_html: None,
                page_no: context.page_no,
                left_percent,
                top_percent,
                width_percent,
                height_percent,
                hidden: false,
                geometry: Some(geometry),
            });
        }
        cursor = cursor.max(segment.end);
    }
    if cursor < raw_text.len() {
        push_cleaned_fragment(&mut parsed.cleaned_text, &raw_text[cursor..]);
    }
    parsed
}

pub(crate) fn apply_region_content(parsed: &mut ParsedOcrPage) {
    let scope_boundaries = sorted_unique_span_starts(&parsed.spans);
    for item in &mut parsed.boxes {
        if let Some(span) = parsed
            .spans
            .iter()
            .find(|span| span.source_region_key == item.source_region_key)
        {
            let start = u64_to_usize_saturating(span.start);
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
        .map_or(text_len, |start| u64_to_usize_saturating(start).min(text_len))
}

fn usize_to_u64_saturating(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn u64_to_usize_saturating(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn push_cleaned_fragment(buffer: &mut String, raw_fragment: &str) {
    let cleaned = remove_marker_tokens(raw_fragment);
    if buffer.chars().last().is_some_and(char::is_whitespace) {
        buffer.push_str(cleaned.trim_start_matches(char::is_whitespace));
        return;
    }
    buffer.push_str(&cleaned);
}

fn marker_geometry_for_box(
    segment: &MarkerSegment,
    index: usize,
    left_percent: f64,
    top_percent: f64,
    width_percent: f64,
    height_percent: f64,
) -> crate::workbench_types::OcrGeometry {
    if index == 0
        && segment.boxes.len() == 1
        && let Some(geometry) = &segment.geometry
    {
        return geometry.clone();
    }
    crate::workbench_types::OcrGeometry::axis_aligned(
        left_percent,
        top_percent,
        width_percent,
        height_percent,
    )
}
