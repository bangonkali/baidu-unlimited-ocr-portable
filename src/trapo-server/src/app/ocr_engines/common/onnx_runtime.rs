use std::path::{Path, PathBuf};

pub(in crate::app::ocr_engines) fn validate_for_native_library(
    native_library_path: &Path,
) -> Result<PathBuf, String> {
    let parent = native_library_path.parent().ok_or_else(|| {
        format!(
            "native OCR library has no parent directory: {}",
            native_library_path.display()
        )
    })?;
    let path = parent.join(library_name());
    if !path.is_file() {
        return Err(format!(
            "ONNX Runtime library is missing for native OCR FFI; expected {} beside {}",
            path.display(),
            native_library_path.display()
        ));
    }
    Ok(path)
}

const fn library_name() -> &'static str {
    if cfg!(windows) {
        "onnxruntime.dll"
    } else if cfg!(target_os = "macos") {
        "libonnxruntime.dylib"
    } else {
        "libonnxruntime.so"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_sibling_onnxruntime_library() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let native_library = temp.path().join(native_library_name());
        std::fs::write(&native_library, b"ffi")?;
        let onnxruntime = temp.path().join(library_name());
        std::fs::write(&onnxruntime, b"ort")?;

        let dependency = validate_for_native_library(&native_library)?;

        assert_eq!(dependency, onnxruntime);
        Ok(())
    }

    #[test]
    fn reports_missing_sibling_onnxruntime_library() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let native_library = temp.path().join(native_library_name());
        std::fs::write(&native_library, b"ffi")?;

        let Err(error) = validate_for_native_library(&native_library) else {
            return Err("ONNX Runtime dependency unexpectedly validated".into());
        };

        assert!(error.contains("ONNX Runtime library is missing"));
        assert!(error.contains(library_name()));
        Ok(())
    }

    const fn native_library_name() -> &'static str {
        if cfg!(windows) {
            "trapo-ocr-ffi.dll"
        } else if cfg!(target_os = "macos") {
            "libtrapo-ocr-ffi.dylib"
        } else {
            "libtrapo-ocr-ffi.so"
        }
    }
}
