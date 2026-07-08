use std::{
    env, fs,
    path::{Path, PathBuf},
};

const ENGINE_ASSET_DIR: &str = "ppocrv6";
const MODEL_MANIFEST: &str = "models/manifest.json";

pub(crate) fn engine_home() -> Result<PathBuf, String> {
    if let Some(home) = env::var_os("TRAPO_PPOCRV6_HOME").map(PathBuf::from)
        && home.is_dir()
    {
        return Ok(home);
    }
    let exe = env::current_exe().map_err(|error| format!("failed to find current exe: {error}"))?;
    let bin_dir = exe
        .parent()
        .ok_or_else(|| "runner executable has no parent directory".to_string())?;
    [
        sibling_cpu_engine_home(bin_dir),
        bin_dir.parent().map(|root| root.join(ENGINE_ASSET_DIR)),
        Some(bin_dir.join(ENGINE_ASSET_DIR)),
        env::current_dir()
            .ok()
            .map(|cwd| cwd.join("thirdparty").join(ENGINE_ASSET_DIR)),
    ]
    .into_iter()
    .flatten()
    .find(|candidate| candidate.is_dir())
    .ok_or_else(|| "packaged PP-OCRv6 native engine directory was not found".to_string())
}

fn sibling_cpu_engine_home(bin_dir: &Path) -> Option<PathBuf> {
    Some(sibling_cpu_runtime_root(bin_dir)?.join(ENGINE_ASSET_DIR))
}

fn sibling_cpu_bin_dir(bin_dir: &Path) -> Option<PathBuf> {
    Some(sibling_cpu_runtime_root(bin_dir)?.join("bin"))
}

fn sibling_cpu_runtime_root(bin_dir: &Path) -> Option<PathBuf> {
    let runtime_root = bin_dir.parent()?;
    let runtime_name = runtime_root.file_name()?.to_str()?;
    let cpu_name = runtime_name
        .strip_suffix("-cuda13")
        .map(|base| format!("{base}-cpu"))?;
    Some(runtime_root.parent()?.join(cpu_name))
}

pub(crate) fn validate_native_assets(home: &Path) -> Result<(), String> {
    if !home.join(MODEL_MANIFEST).is_file() {
        return Err(format!(
            "PP-OCRv6 model manifest is missing: {}",
            home.join(MODEL_MANIFEST).display()
        ));
    }
    if contains_python_runtime_assets(home) {
        return Err(format!(
            "PP-OCRv6 runtime must be native-only; remove Python/.venv assets from {}",
            home.display()
        ));
    }
    let _ = ffi_library_path(home)?;
    Ok(())
}

pub(crate) fn ffi_library_path(home: &Path) -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("TRAPO_OCR_FFI_LIBRARY").map(PathBuf::from)
        && path.is_file()
    {
        return Ok(path);
    }
    let exe = env::current_exe().map_err(|error| format!("failed to find current exe: {error}"))?;
    let bin_dir = exe
        .parent()
        .ok_or_else(|| "runner executable has no parent directory".to_string())?;
    let names = ffi_library_names();
    [
        Some(home.join("bin")),
        sibling_cpu_bin_dir(bin_dir),
        Some(bin_dir.to_path_buf()),
    ]
    .into_iter()
    .flatten()
    .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
    .find(|candidate| candidate.is_file())
    .ok_or_else(|| {
        format!(
            "trapo-ocr-ffi native library is missing; expected one of {:?} in {}",
            names,
            home.join("bin").display()
        )
    })
}

const fn ffi_library_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["trapo-ocr-ffi.dll"]
    } else if cfg!(target_os = "macos") {
        &["libtrapo-ocr-ffi.dylib"]
    } else {
        &["libtrapo-ocr-ffi.so"]
    }
}

fn contains_python_runtime_assets(home: &Path) -> bool {
    contains_forbidden_asset(home, home)
}

fn contains_forbidden_asset(root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if is_forbidden_ppocrv6_asset(relative) {
        return true;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return true;
    };
    if !metadata.is_dir() {
        return false;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return true;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            return true;
        };
        if contains_forbidden_asset(root, &entry.path()) {
            return true;
        }
    }
    false
}

fn is_forbidden_ppocrv6_asset(relative: &Path) -> bool {
    let filename = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if filename.starts_with("trapo_ppocrv6_engine") || has_forbidden_extension(relative) {
        return true;
    }
    relative.components().any(|component| {
        let name = component
            .as_os_str()
            .to_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        matches!(
            name.as_str(),
            ".venv" | ".paddlex" | "__pycache__" | "build" | "ppocrv6"
        ) || name.starts_with("trapo_ppocrv6_engine")
    })
}

fn has_forbidden_extension(relative: &Path) -> bool {
    relative
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "py" | "pyc" | "pyo" | "pyd" | "spec"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{is_forbidden_ppocrv6_asset, sibling_cpu_bin_dir, sibling_cpu_engine_home};
    use std::path::{Path, PathBuf};

    #[test]
    fn rejects_python_runtime_artifacts_at_any_depth() {
        for path in [
            "trapo_ppocrv6_engine.py",
            "build/trapo_ppocrv6_engine/localpycs/struct.pyc",
            "ppocrv6/models/manifest.json",
            ".paddlex/temp/cache",
            "models/__pycache__/old.pyc",
            ".venv/Scripts/python.exe",
        ] {
            assert!(is_forbidden_ppocrv6_asset(Path::new(path)));
        }
    }

    #[test]
    fn allows_native_model_assets() {
        for path in [
            "models/manifest.json",
            "models/text_detection/inference.onnx",
            "models/text_recognition/inference.yml",
        ] {
            assert!(!is_forbidden_ppocrv6_asset(Path::new(path)));
        }
    }

    #[test]
    fn cuda_runtime_prefers_sibling_cpu_engine_home() {
        let bin_dir = PathBuf::from("runtime/windows-x86_64-cuda13/bin");

        assert_eq!(
            sibling_cpu_engine_home(&bin_dir),
            Some(PathBuf::from("runtime/windows-x86_64-cpu/ppocrv6"))
        );
        assert_eq!(
            sibling_cpu_bin_dir(&bin_dir),
            Some(PathBuf::from("runtime/windows-x86_64-cpu/bin"))
        );
    }
}
