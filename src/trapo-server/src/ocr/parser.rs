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
        let clean_end = parsed.cleaned_text.len() as u64;
        for box_points in &segment.boxes {
            let region_id = region_id_for(context, segment, box_points);
            parsed.spans.push(TextRegionSpan {
                region_id: region_id.clone(),
                page_no: context.page_no,
                start: clean_start,
                end: clean_end,
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
    for item in &mut parsed.boxes {
        if let Some(span) = parsed
            .spans
            .iter()
            .find(|span| span.region_id == item.region_id)
        {
            let start = span.start as usize;
            let end = span.end as usize;
            if start <= end && end <= parsed.cleaned_text.len() {
                item.content_markdown = parsed.cleaned_text[start..end].to_string();
            }
        }
    }
}
