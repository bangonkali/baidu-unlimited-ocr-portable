use serde_json::{Value, json};

pub(super) fn native_json_to_marker_text(native_json: &str) -> Result<String, String> {
    let payload = serde_json::from_str::<Value>(native_json)
        .map_err(|error| format!("native OCR returned invalid JSON: {error}"))?;
    let mut lines = Vec::new();
    for page in payload
        .get("pages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        append_page_lines(page, &mut lines);
    }
    if lines.is_empty() {
        return Ok(payload
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string());
    }
    Ok(lines.join("\n"))
}

fn append_page_lines(page: &Value, output: &mut Vec<String>) {
    let width = page.get("width").and_then(Value::as_f64).unwrap_or(1.0);
    let height = page.get("height").and_then(Value::as_f64).unwrap_or(1.0);
    let Some(lines) = page.get("lines").and_then(Value::as_array) else {
        return;
    };
    for line in lines {
        let text = line.get("text").and_then(Value::as_str).unwrap_or_default();
        if text.trim().is_empty() {
            continue;
        }
        let Some((x1, y1, x2, y2)) = line_bounds(line, width, height) else {
            output.push(text.to_string());
            continue;
        };
        let geometry = line_geometry(line, width, height)
            .map_or_else(String::new, |value| format!("<|geom|>{value}<|/geom|>"));
        output.push(format!(
            "<|ref|>{}<|/ref|><|det|>[[{x1},{y1},{x2},{y2}]]<|/det|>{geometry} {}",
            marker_safe(text),
            text
        ));
    }
}

fn line_bounds(line: &Value, page_width: f64, page_height: f64) -> Option<(u32, u32, u32, u32)> {
    if let Some(bounds) = line.get("boundingBox") {
        return Some((
            pixel_to_marker(bounds.get("left")?.as_f64()?, page_width),
            pixel_to_marker(bounds.get("top")?.as_f64()?, page_height),
            pixel_to_marker(bounds.get("right")?.as_f64()?, page_width),
            pixel_to_marker(bounds.get("bottom")?.as_f64()?, page_height),
        ));
    }
    let points = line.get("polygon")?.as_array()?;
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in points {
        let x = point.get("x").and_then(Value::as_f64)?;
        let y = point.get("y").and_then(Value::as_f64)?;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    min_x.is_finite().then(|| {
        (
            pixel_to_marker(min_x, page_width),
            pixel_to_marker(min_y, page_height),
            pixel_to_marker(max_x, page_width),
            pixel_to_marker(max_y, page_height),
        )
    })
}

fn line_geometry(line: &Value, page_width: f64, page_height: f64) -> Option<String> {
    let points = line
        .get("polygon")?
        .as_array()?
        .iter()
        .map(|point| {
            Some(json!({
                "x": pixel_to_percent(point.get("x")?.as_f64()?, page_width),
                "y": pixel_to_percent(point.get("y")?.as_f64()?, page_height),
            }))
        })
        .collect::<Option<Vec<_>>>()?;
    if points.len() < 3 {
        return None;
    }
    serde_json::to_string(&json!({
        "kind": if points.len() == 4 { "rotated_quad" } else { "polygon" },
        "points": points,
        "rotation_degrees": rotation_degrees(&points),
        "layer_id": "source",
        "coordinate_space": "page_percent",
        "bounds": geometry_bounds(&points)?,
    }))
    .ok()
}

fn geometry_bounds(points: &[Value]) -> Option<Value> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in points {
        let x = point.get("x").and_then(Value::as_f64)?;
        let y = point.get("y").and_then(Value::as_f64)?;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    min_x.is_finite().then(|| {
        json!({
            "left": min_x,
            "top": min_y,
            "width": (max_x - min_x).max(0.0),
            "height": (max_y - min_y).max(0.0),
        })
    })
}

fn rotation_degrees(points: &[Value]) -> Option<f64> {
    let first = points.first()?;
    let second = points.get(1)?;
    let x1 = first.get("x").and_then(Value::as_f64)?;
    let y1 = first.get("y").and_then(Value::as_f64)?;
    let x2 = second.get("x").and_then(Value::as_f64)?;
    let y2 = second.get("y").and_then(Value::as_f64)?;
    Some((y2 - y1).atan2(x2 - x1).to_degrees())
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "OCR marker coordinates are clamped before converting to the 0..999 integer space"
)]
fn pixel_to_marker(value: f64, dimension: f64) -> u32 {
    if dimension <= 0.0 {
        return 0;
    }
    ((value / dimension * 999.0).round().clamp(0.0, 999.0)) as u32
}

fn pixel_to_percent(value: f64, dimension: f64) -> f64 {
    if dimension <= 0.0 {
        return 0.0;
    }
    (value / dimension * 100.0).clamp(0.0, 100.0)
}

fn marker_safe(value: &str) -> String {
    value.replace("<|", "< |").replace("|>", "| >")
}
