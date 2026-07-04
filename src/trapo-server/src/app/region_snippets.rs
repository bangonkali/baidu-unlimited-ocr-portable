use image::GenericImageView;

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
            let Some((left, top, width, height)) = crop_bounds(item, image_width, image_height)
            else {
                continue;
            };
            let snippet = page.crop_imm(left, top, width, height);
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

fn crop_bounds(item: &OverlayBox, image_width: u32, image_height: u32) -> Option<(u32, u32, u32, u32)> {
    let left = percent_to_pixel(item.left_percent, image_width);
    let top = percent_to_pixel(item.top_percent, image_height);
    let right = percent_to_pixel(item.left_percent + item.width_percent, image_width);
    let bottom = percent_to_pixel(item.top_percent + item.height_percent, image_height);
    let width = right.saturating_sub(left).max(1);
    let height = bottom.saturating_sub(top).max(1);
    (left < image_width && top < image_height).then_some((
        left,
        top,
        width.min(image_width.saturating_sub(left)),
        height.min(image_height.saturating_sub(top)),
    ))
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "region crop percentages are clamped to the unsigned image dimensions before rounding"
)]
fn percent_to_pixel(percent: f64, dimension: u32) -> u32 {
    ((percent.clamp(0.0, 100.0) / 100.0) * f64::from(dimension)).round() as u32
}

fn region_snippet_url(file_hash: &str, region_id: &str) -> String {
    format!("/api/documents/{file_hash}/regions/{region_id}/snippet")
}

fn markdown_alt(label: &str) -> String {
    label.replace(['[', ']'], " ").trim().to_string()
}
