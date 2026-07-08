use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct OcrGeometry {
    #[serde(default)]
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) points: Vec<OcrGeometryPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) rotation_degrees: Option<f64>,
    #[serde(default)]
    pub(crate) layer_id: String,
    #[serde(default)]
    pub(crate) coordinate_space: String,
    #[serde(default)]
    pub(crate) bounds: OcrGeometryBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct OcrGeometryPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct OcrGeometryBounds {
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl OcrGeometry {
    pub(crate) fn axis_aligned(left: f64, top: f64, width: f64, height: f64) -> Self {
        let right = left + width;
        let bottom = top + height;
        Self {
            kind: "axis_aligned".to_string(),
            points: vec![
                OcrGeometryPoint { x: left, y: top },
                OcrGeometryPoint { x: right, y: top },
                OcrGeometryPoint {
                    x: right,
                    y: bottom,
                },
                OcrGeometryPoint { x: left, y: bottom },
            ],
            rotation_degrees: None,
            layer_id: "source".to_string(),
            coordinate_space: "page_percent".to_string(),
            bounds: OcrGeometryBounds {
                left,
                top,
                width,
                height,
            },
        }
    }

    pub(crate) fn from_storage_json(
        value: &str,
        stored_kind: &str,
        bounds: OcrGeometryBounds,
        rotation_degrees: Option<f64>,
    ) -> Self {
        let fallback = Self::axis_aligned(bounds.left, bounds.top, bounds.width, bounds.height);
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "{}" {
            return fallback.with_storage_defaults(stored_kind, rotation_degrees);
        }
        match serde_json::from_str::<Self>(trimmed) {
            Ok(parsed) if !parsed.points.is_empty() => {
                parsed.with_storage_defaults(stored_kind, rotation_degrees)
            }
            _ => fallback.with_storage_defaults(stored_kind, rotation_degrees),
        }
    }

    fn with_storage_defaults(mut self, stored_kind: &str, rotation_degrees: Option<f64>) -> Self {
        if self.kind.is_empty() {
            self.kind = match stored_kind {
                "rotated_quad" | "polygon" | "axis_aligned" => stored_kind.to_string(),
                _ => "axis_aligned".to_string(),
            };
        }
        if self.layer_id.is_empty() {
            self.layer_id = "source".to_string();
        }
        if self.coordinate_space.is_empty() {
            self.coordinate_space = "page_percent".to_string();
        }
        if self.rotation_degrees.is_none() {
            self.rotation_degrees = rotation_degrees;
        }
        if self.bounds.width <= 0.0 || self.bounds.height <= 0.0 {
            self.bounds = self.computed_bounds();
        }
        self
    }

    fn computed_bounds(&self) -> OcrGeometryBounds {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for point in &self.points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return OcrGeometryBounds::default();
        }
        OcrGeometryBounds {
            left: min_x,
            top: min_y,
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        }
    }
}
