use std::path::{Path, PathBuf};

use image::{GenericImageView, ImageFormat};
use pdfium::{PdfiumDocument, PdfiumRenderConfig};

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub(crate) struct RenderedPage {
    pub(crate) page_no: u32,
    pub(crate) image_path: PathBuf,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct PdfRenderer {
    cache_dir: PathBuf,
    pdfium_dir: Option<PathBuf>,
    dpi: u32,
}

impl PdfRenderer {
    pub(crate) fn new(
        cache_dir: impl Into<PathBuf>,
        pdfium_dir: Option<PathBuf>,
        dpi: u32,
    ) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            pdfium_dir,
            dpi,
        }
    }

    pub(crate) fn render_pdf(&self, file_hash: &str, pdf_path: &Path) -> Result<Vec<RenderedPage>> {
        let Some(pdfium_dir) = &self.pdfium_dir else {
            return Err(AppError::Internal(
                "PDFium runtime was not found; set TRAPO_PDFIUM_DIR or run from a packaged Trapo workbench".to_string(),
            ));
        };
        pdfium::set_library_location(&pdfium_library_location(pdfium_dir));
        let document = PdfiumDocument::new_from_path(native_external_path(pdf_path), None) // skylos: ignore[SKY-D215] pdf_path is from validated local ingest discovery.
            .map_err(|error| {
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
            let bitmap = page
                .render(
                    &PdfiumRenderConfig::new()
                        .with_width(i32::try_from(width_px).unwrap_or(i32::MAX)),
                )
                .map_err(|error| {
                    AppError::Internal(format!("failed to render PDF page: {error:?}"))
                })?;
            let image_path = page_dir.join(format!("page-{}.png", index + 1));
            bitmap
                .save(&native_external_path_string(&image_path), ImageFormat::Png)
                .map_err(|error| {
                    AppError::Internal(format!("failed to save PDF preview: {error:?}"))
                })?;
            pages.push(RenderedPage {
                page_no: u32::try_from(index).unwrap_or(0) + 1,
                image_path,
                width_px: i32_to_u32_saturating(bitmap.width()),
                height_px: i32_to_u32_saturating(bitmap.height()),
            });
        }
        Ok(pages)
    }

    pub(crate) fn image_page(image_path: &Path) -> Result<RenderedPage> {
        let image = image::open(native_external_path(image_path)) // skylos: ignore[SKY-D215] image_path is from validated local ingest discovery.
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

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "PDF point sizes are clamped to the supported unsigned pixel range before rounding"
)]
fn points_to_pixels(points: f32, dpi: u32) -> u32 {
    ((f64::from(points) / 72.0) * f64::from(dpi))
        .round()
        .clamp(1.0, f64::from(u32::MAX)) as u32
}

fn i32_to_u32_saturating(value: i32) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(u32::MAX)
}

fn pdfium_library_location(path: &Path) -> String {
    native_external_path_string(path)
}

fn native_external_path_string(path: &Path) -> String {
    native_external_path(path).to_string_lossy().into_owned()
}

fn native_external_path(path: &Path) -> PathBuf {
    strip_windows_verbatim_prefix(path).unwrap_or_else(|| path.to_path_buf())
}

#[cfg(windows)]
fn strip_windows_verbatim_prefix(path: &Path) -> Option<PathBuf> {
    let value = path.as_os_str().to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        return Some(PathBuf::from(format!(r"\\{rest}")));
    }
    let rest = value.strip_prefix(r"\\?\")?;
    if rest.as_bytes().get(1) == Some(&b':') {
        Some(PathBuf::from(rest))
    } else {
        None
    }
}

#[cfg(not(windows))]
const fn strip_windows_verbatim_prefix(_path: &Path) -> Option<PathBuf> {
    None
}

#[must_use]
pub(crate) fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_external_path_keeps_relative_paths() {
        assert_eq!(
            native_external_path(Path::new("docs/sample.pdf")),
            PathBuf::from("docs/sample.pdf")
        );
    }

    #[cfg(windows)]
    #[test]
    fn native_external_path_strips_windows_verbatim_drive_prefix() {
        assert_eq!(
            native_external_path(Path::new(r"\\?\C:\docs\sample.pdf")),
            PathBuf::from(r"C:\docs\sample.pdf")
        );
    }

    #[cfg(windows)]
    #[test]
    fn native_external_path_strips_windows_verbatim_unc_prefix() {
        assert_eq!(
            native_external_path(Path::new(r"\\?\UNC\server\share\sample.pdf")),
            PathBuf::from(r"\\server\share\sample.pdf")
        );
    }
}
