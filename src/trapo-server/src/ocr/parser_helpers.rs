fn collect_segments(raw: &str) -> Vec<MarkerSegment> {
    static REF_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        compiled_regex(r"(?s)<\|ref\|>(.*?)<\|/ref\|>\s*<\|det\|>\s*(.*?)\s*<\|/det\|>")
    });
    static DET_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| compiled_regex(r"(?s)<\|det\|>\s*(.*?)\s*<\|/det\|>"));
    let mut segments = Vec::new();
    for capture in REF_PATTERN.captures_iter(raw) {
        let Some(matched) = capture.get(0) else {
            continue;
        };
        let boxes = parse_box_points(capture.get(2).map(|item| item.as_str()).unwrap_or_default());
        if boxes.is_empty() {
            continue;
        }
        segments.push(MarkerSegment {
            start: matched.start(),
            end: matched.end(),
            label: trim(capture.get(1).map(|item| item.as_str()).unwrap_or_default()),
            boxes,
        });
    }
    for capture in DET_PATTERN.captures_iter(raw) {
        let Some(matched) = capture.get(0) else {
            continue;
        };
        if span_is_inside(matched.start(), matched.end(), &segments) {
            continue;
        }
        let content = trim(capture.get(1).map(|item| item.as_str()).unwrap_or_default());
        let Some(bracket_at) = content.find('[') else {
            continue;
        };
        let boxes = parse_box_points(&content[bracket_at..]);
        if boxes.is_empty() {
            continue;
        }
        let label = trim(&content[..bracket_at]);
        segments.push(MarkerSegment {
            start: matched.start(),
            end: matched.end(),
            label: if label.is_empty() {
                "det".to_string()
            } else {
                label
            },
            boxes,
        });
    }
    segments.sort_by_key(|segment| segment.start);
    segments
}

fn parse_box_points(raw: &str) -> Vec<BoxPoints> {
    static BOX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        compiled_regex(
            r"\[\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\]",
        )
    });
    BOX_PATTERN
        .captures_iter(raw)
        .filter_map(|capture| {
            let mut points = BoxPoints {
                x1: capture.get(1)?.as_str().parse().ok()?,
                y1: capture.get(2)?.as_str().parse().ok()?,
                x2: capture.get(3)?.as_str().parse().ok()?,
                y2: capture.get(4)?.as_str().parse().ok()?,
            };
            if points.x2 < points.x1 {
                std::mem::swap(&mut points.x1, &mut points.x2);
            }
            if points.y2 < points.y1 {
                std::mem::swap(&mut points.y1, &mut points.y2);
            }
            points.x1 = points.x1.clamp(0.0, 999.0);
            points.y1 = points.y1.clamp(0.0, 999.0);
            points.x2 = points.x2.clamp(0.0, 999.0);
            points.y2 = points.y2.clamp(0.0, 999.0);
            Some(points)
        })
        .collect()
}

fn region_source_key_for(
    context: &ParseContext,
    segment: &MarkerSegment,
    box_points: &BoxPoints,
) -> String {
    let key = format!(
        "{}|{}|{}|{}|{}:{}|{}|{},{},{},{}",
        context.file_hash,
        context.page_no,
        context.engine_id,
        context.profile_id,
        segment.start,
        segment.end,
        segment.label,
        box_points.x1,
        box_points.y1,
        box_points.x2,
        box_points.y2
    );
    format!("src_{}", &region_hash_key([key])[4..])
}

fn span_is_inside(start: usize, end: usize, segments: &[MarkerSegment]) -> bool {
    segments
        .iter()
        .any(|segment| start >= segment.start && end <= segment.end)
}

fn remove_marker_tokens(value: &str) -> String {
    static MARKER_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| compiled_regex(r"<\|/?(?:ref|det)\|>"));
    MARKER_PATTERN.replace_all(value, "").into_owned()
}

fn trim(value: &str) -> String {
    value.trim_matches([' ', '\t', '\r', '\n']).to_string()
}

fn format_prompt(prompt: &str, media_placement: &str) -> String {
    if prompt.contains("<image>") {
        return prompt.to_string();
    }
    match media_placement {
        "prefix-tight" => format!("<image>{prompt}"),
        "suffix-newline" => format!("{prompt}\n<image>"),
        _ => format!("<image>\n{prompt}"),
    }
}
