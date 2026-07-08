use image::{DynamicImage, GenericImageView};

use crate::workbench_types::OverlayBox;

impl AppState {
    fn region_snippet_path(&self, file_hash: &str, region_id: &str) -> PathBuf {
        self.inner
            .config
            .cache_dir
            .join("region-snippets")
            .join(file_hash)
            .join(format!("{region_id}.png"))
    }

    fn region_snippets_dir(&self, file_hash: &str) -> PathBuf {
        self.inner
            .config
            .cache_dir
            .join("region-snippets")
            .join(file_hash)
    }

    fn write_image_region_snippets(
        &self,
        file_hash: &str,
        page_image: &Path,
        boxes: &mut [OverlayBox],
    ) -> Result<()> {
        if !boxes.iter().any(is_image_region) {
            return Ok(());
        }
        let page = match image::open(page_image) {
            Ok(page) => page,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    path = %page_image.display(),
                    "failed to read page image for region snippets"
                );
                return Ok(());
            }
        };
        std::fs::create_dir_all(self.region_snippets_dir(file_hash))?;
        let (image_width, image_height) = page.dimensions();
        for item in boxes.iter_mut().filter(|item| is_image_region(item)) {
            let Some(crop) = crop_bounds(item, image_width, image_height) else {
                continue;
            };
            let snippet = crop_region(&page, item, crop);
            let path = self.region_snippet_path(file_hash, &item.region_id);
            snippet
                .save_with_format(&path, image::ImageFormat::Png)
                .map_err(|error| {
                    AppError::Internal(format!("failed to save region snippet: {error}"))
                })?;
            let url = region_snippet_url(file_hash, &item.region_id);
            item.content_markdown = format!("![{}]({url})", markdown_alt(&item.label));
            item.content_html = Some("image-snippet".to_string());
        }
        Ok(())
    }

    pub(crate) async fn region_snippet_path_for_request(
        &self,
        file_hash: &str,
        region_id: &str,
    ) -> Option<PathBuf> {
        let has_region = {
            let state = self.inner.state.lock().await;
            let has_region = state.documents.get(file_hash).is_some_and(|document| {
                document.pages.iter().any(|page| {
                    page.boxes
                        .iter()
                        .any(|item| item.region_id == region_id && is_image_region(item))
                })
            });
            drop(state);
            has_region
        };
        let path = self.region_snippet_path(file_hash, region_id);
        (has_region && path.is_file()).then_some(path)
    }
}

fn is_image_region(item: &OverlayBox) -> bool {
    item.content_html.as_deref() == Some("image-snippet") || text_looks_image_like(item)
}

fn text_looks_image_like(item: &OverlayBox) -> bool {
    let text = format!("{} {}", item.label, item.content_markdown).to_lowercase();
    ["image", "figure", "picture", "diagram", "chart", "photo"]
        .iter()
        .any(|needle| text.contains(needle))
}

#[derive(Clone, Copy)]
struct CropWindow {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    image_width: u32,
    image_height: u32,
}

fn crop_bounds(item: &OverlayBox, image_width: u32, image_height: u32) -> Option<CropWindow> {
    let bounds = region_bounds_percent(item);
    let left = percent_to_pixel(bounds.0, image_width);
    let top = percent_to_pixel(bounds.1, image_height);
    let right = percent_to_pixel(bounds.0 + bounds.2, image_width);
    let bottom = percent_to_pixel(bounds.1 + bounds.3, image_height);
    let width = right.saturating_sub(left).max(1);
    let height = bottom.saturating_sub(top).max(1);
    (left < image_width && top < image_height).then_some(CropWindow {
        left,
        top,
        width: width.min(image_width.saturating_sub(left)),
        height: height.min(image_height.saturating_sub(top)),
        image_width,
        image_height,
    })
}

fn crop_region(page: &DynamicImage, item: &OverlayBox, crop: CropWindow) -> DynamicImage {
    let snippet = page.crop_imm(crop.left, crop.top, crop.width, crop.height);
    let Some(points) = polygon_points_for_crop(item, crop) else {
        return snippet;
    };
    let mut rgba = snippet.to_rgba8();
    for y in 0..rgba.height() {
        for x in 0..rgba.width() {
            if !point_in_polygon(f64::from(x) + 0.5, f64::from(y) + 0.5, &points) {
                rgba.get_pixel_mut(x, y).0[3] = 0;
            }
        }
    }
    DynamicImage::ImageRgba8(rgba)
}

fn region_bounds_percent(item: &OverlayBox) -> (f64, f64, f64, f64) {
    let Some(geometry) = item.geometry.as_ref() else {
        return (
            item.left_percent,
            item.top_percent,
            item.width_percent,
            item.height_percent,
        );
    };
    if geometry.coordinate_space != "page_percent" || geometry.points.is_empty() {
        return (
            item.left_percent,
            item.top_percent,
            item.width_percent,
            item.height_percent,
        );
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in &geometry.points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        return (min_x, min_y, (max_x - min_x).max(0.0), (max_y - min_y).max(0.0));
    }
    (
        item.left_percent,
        item.top_percent,
        item.width_percent,
        item.height_percent,
    )
}

fn polygon_points_for_crop(
    item: &OverlayBox,
    crop: CropWindow,
) -> Option<Vec<(f64, f64)>> {
    let geometry = item.geometry.as_ref()?;
    if geometry.coordinate_space != "page_percent"
        || !matches!(geometry.kind.as_str(), "rotated_quad" | "polygon")
        || geometry.points.len() < 3
    {
        return None;
    }
    Some(
        geometry
            .points
            .iter()
            .map(|point| {
                (
                    percent_to_float_pixel(point.x, crop.image_width) - f64::from(crop.left),
                    percent_to_float_pixel(point.y, crop.image_height) - f64::from(crop.top),
                )
            })
            .collect(),
    )
}

fn point_in_polygon(x: f64, y: f64, points: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let mut previous = points.len() - 1;
    for current in 0..points.len() {
        let (current_x, current_y) = points[current];
        let (previous_x, previous_y) = points[previous];
        let crosses = (current_y > y) != (previous_y > y);
        if crosses {
            let intersection_x =
                (previous_x - current_x) * (y - current_y) / (previous_y - current_y) + current_x;
            if x < intersection_x {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "region crop percentages are clamped to the unsigned image dimensions before rounding"
)]
fn percent_to_pixel(percent: f64, dimension: u32) -> u32 {
    ((percent.clamp(0.0, 100.0) / 100.0) * f64::from(dimension)).round() as u32
}

fn percent_to_float_pixel(percent: f64, dimension: u32) -> f64 {
    (percent.clamp(0.0, 100.0) / 100.0) * f64::from(dimension)
}

fn region_snippet_url(file_hash: &str, region_id: &str) -> String {
    format!("/api/documents/{file_hash}/regions/{region_id}/snippet")
}

fn markdown_alt(label: &str) -> String {
    label.replace(['[', ']'], " ").trim().to_string()
}
