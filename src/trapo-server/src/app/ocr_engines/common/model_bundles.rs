use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(in crate::app::ocr_engines) struct ModelBundleCheck {
    label: &'static str,
    missing: Vec<PathBuf>,
}

impl ModelBundleCheck {
    const fn new(label: &'static str, missing: Vec<PathBuf>) -> Self {
        Self { label, missing }
    }

    pub(in crate::app::ocr_engines) fn ensure_available(self) -> Result<(), String> {
        if self.missing.is_empty() {
            return Ok(());
        }
        let missing = self
            .missing
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        Err(format!(
            "{} model bundle is incomplete. Missing: {missing}",
            self.label
        ))
    }
}

pub(in crate::app::ocr_engines) fn ppocrv6(root: &Path) -> ModelBundleCheck {
    required_files("PP-OCRv6", [root.join("models").join("manifest.json")])
}

pub(in crate::app::ocr_engines) fn paddleocr_vl_1_6(
    root: &Path,
    vl_model_path: &Path,
    vl_mmproj_path: &Path,
) -> ModelBundleCheck {
    let layout_model_path = first_existing_path([
        root.join("layout_detection").join("inference.onnx"),
        root.join("layout_detection").join("PP-DocLayoutV3.onnx"),
    ]);
    required_files(
        "PaddleOCR-VL-1.6",
        [
            root.join("manifest.json"),
            layout_model_path,
            vl_model_path.to_path_buf(),
            vl_mmproj_path.to_path_buf(),
        ],
    )
}

#[allow(
    dead_code,
    reason = "Document Markdown Rust resolver wiring lands after the native parity validator port"
)]
pub(in crate::app::ocr_engines) fn document_markdown(root: &Path) -> ModelBundleCheck {
    required_files(
        "Document Markdown",
        [
            root.join("manifest.json"),
            root.join("doc_orientation").join("inference.onnx"),
            root.join("config").join("config.json"),
            root.join("config").join("tokenizer.json"),
            root.join("onnx").join("vision_encoder_q4.onnx"),
            root.join("onnx").join("embed_tokens_q4.onnx"),
            root.join("onnx").join("decoder_model_merged_q4.onnx"),
            root.join("onnx").join("vision_encoder_q4.onnx_data"),
            root.join("onnx").join("embed_tokens_q4.onnx_data"),
            root.join("onnx").join("embed_tokens_q4.onnx_data_1"),
            root.join("onnx").join("decoder_model_merged_q4.onnx_data"),
            root.join("onnx")
                .join("decoder_model_merged_q4.onnx_data_1"),
        ],
    )
}

fn required_files<const N: usize>(label: &'static str, paths: [PathBuf; N]) -> ModelBundleCheck {
    ModelBundleCheck::new(
        label,
        paths.into_iter().filter(|path| !path.is_file()).collect(),
    )
}

fn first_existing_path<const N: usize>(paths: [PathBuf; N]) -> PathBuf {
    paths
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .unwrap_or_else(|| paths.into_iter().next().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppocrv6_reports_missing_manifest() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let Err(error) = ppocrv6(temp.path()).ensure_available() else {
            return Err("PP-OCRv6 bundle unexpectedly validated".into());
        };

        assert!(error.contains("PP-OCRv6 model bundle is incomplete"));
        assert!(error.contains("models"));
        assert!(error.contains("manifest.json"));
        Ok(())
    }

    #[test]
    fn paddleocr_vl_accepts_layout_candidate_and_external_models()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("paddleocr_vl_1_6");
        std::fs::create_dir_all(root.join("layout_detection"))?;
        std::fs::write(root.join("manifest.json"), b"{}")?;
        std::fs::write(
            root.join("layout_detection").join("PP-DocLayoutV3.onnx"),
            b"onnx",
        )?;
        let vl_model = temp.path().join("PaddleOCR-VL-1.6-GGUF.gguf");
        let mmproj = temp.path().join("PaddleOCR-VL-1.6-GGUF-mmproj.gguf");
        std::fs::write(&vl_model, b"model")?;
        std::fs::write(&mmproj, b"mmproj")?;

        paddleocr_vl_1_6(&root, &vl_model, &mmproj).ensure_available()?;
        Ok(())
    }

    #[test]
    fn document_markdown_reports_model_data_files() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let Err(error) = document_markdown(temp.path()).ensure_available() else {
            return Err("Document Markdown bundle unexpectedly validated".into());
        };

        assert!(error.contains("Document Markdown model bundle is incomplete"));
        assert!(error.contains("decoder_model_merged_q4.onnx_data_1"));
        Ok(())
    }
}
