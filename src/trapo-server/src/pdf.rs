use std::path::{Path, PathBuf};

use image::{GenericImageView, ImageFormat};
use pdfium::{PdfiumDocument, PdfiumRenderConfig};

use crate::{
    error::{AppError, Result},
    scanner::generic_path,
};

#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub page_no: u32,
    pub image_path: PathBuf,
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Debug, Clone)]
pub struct PdfRenderer {
    cache_dir: PathBuf,
    pdfium_dir: Option<PathBuf>,
    dpi: u32,
}

impl PdfRenderer {
    pub fn new(cache_dir: impl Into<PathBuf>, pdfium_dir: Option<PathBuf>, dpi: u32) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            pdfium_dir,
            dpi,
        }
    }

    pub fn render_pdf(&self, file_hash: &str, pdf_path: &Path) -> Result<Vec<RenderedPage>> {
        if let Some(pdfium_dir) = &self.pdfium_dir {
            pdfium::set_library_location(&generic_path(pdfium_dir));
        }
        let document =
            PdfiumDocument::new_from_path(generic_path(pdf_path), None).map_err(|error| {
                AppError::Internal(format!("failed to open PDF with PDFium: {error:?}"))
            })?;
        let page_dir = self.cache_dir.join("previews").join(file_hash);
        std::fs::create_dir_all(&page_dir)?;
        let mut pages = Vec::new();
        for index in 0..document.page_count() {
            let page = document.page(index).map_err(|error| {
                AppError::Internal(format!("failed to read PDF page: {error:?}"))
            })?;
            let width_px = points_to_pixels(page.width(), self.dpi).max(1);
            let height_px = points_to_pixels(page.height(), self.dpi).max(1);
            let bitmap = page
                .render(
                    &PdfiumRenderConfig::new()
                        .with_width(width_px as i32)
                        .with_height(height_px as i32),
                )
                .map_err(|error| {
                    AppError::Internal(format!("failed to render PDF page: {error:?}"))
                })?;
            let image_path = page_dir.join(format!("page-{}.png", index + 1));
            bitmap
                .save(&generic_path(&image_path), ImageFormat::Png)
                .map_err(|error| {
                    AppError::Internal(format!("failed to save PDF preview: {error:?}"))
                })?;
            pages.push(RenderedPage {
                page_no: u32::try_from(index).unwrap_or(0) + 1,
                image_path,
                width_px: bitmap.width() as u32,
                height_px: bitmap.height() as u32,
            });
        }
        Ok(pages)
    }

    pub fn image_page(&self, image_path: &Path) -> Result<RenderedPage> {
        let image = image::open(image_path)
            .map_err(|error| AppError::BadRequest(format!("failed to read image: {error}")))?;
        let (width_px, height_px) = image.dimensions();
        Ok(RenderedPage {
            page_no: 1,
            image_path: image_path.to_path_buf(),
            width_px,
            height_px,
        })
    }
}

fn points_to_pixels(points: f32, dpi: u32) -> u32 {
    ((points / 72.0) * dpi as f32).round() as u32
}

pub fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}
