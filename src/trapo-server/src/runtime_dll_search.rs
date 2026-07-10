//! Ensure packaged OCR/CUDA runtime `bin` directories are on `PATH` for in-process
//! FFI (ORT CUDA EP, cuDNN stubs, llama CUDA).
//!
//! `trapo-server.exe` is often started without `trapo-server.cmd`, so PATH may
//! not include `thirdparty/uocr-runtime/*/bin`. cuDNN 9 resolves sibling DLLs
//! by basename via the standard search path; without those directories on PATH it
//! prints `Invalid handle. Cannot load symbol cudnnCreate` and CUDA EP init
//! fails.

#![allow(unsafe_code)]

use std::{
    env,
    path::{Path, PathBuf},
    sync::Once,
};

static REGISTER: Once = Once::new();

/// Prepend every packaged `uocr-runtime/*/bin` to `PATH` (idempotent).
pub fn ensure_runtime_dll_search_paths() {
    REGISTER.call_once(|| {
        let bins = discover_runtime_bin_dirs();
        if bins.is_empty() {
            return;
        }
        prepend_path_dirs(&bins);
    });
}

fn discover_runtime_bin_dirs() -> Vec<PathBuf> {
    // Only search beside the running executable. Using cwd would pick up
    // developer `dist/` or repo runtimes during cargo tests and falsely mark
    // native runners as installed.
    let Some(root) = env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    else {
        return Vec::new();
    };
    let runtime_root = root.join("thirdparty").join("uocr-runtime");
    let Ok(entries) = std::fs::read_dir(&runtime_root) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path().join("bin"))
        .filter(|path| path.is_dir())
        .collect();
    // Prefer CUDA/accelerator bins ahead of CPU so basename DLL loads pick
    // the GPU redistributables when both runtimes are packaged.
    dirs.sort_by(|left, right| {
        runtime_bin_priority(right)
            .cmp(&runtime_bin_priority(left))
            .then_with(|| left.cmp(right))
    });
    dirs
}

fn runtime_bin_priority(path: &Path) -> u8 {
    let name = path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if name.contains("cuda") {
        2
    } else {
        u8::from(name.contains("rocm") || name.contains("metal"))
    }
}

fn prepend_path_dirs(dirs: &[PathBuf]) {
    let mut paths: Vec<PathBuf> = dirs.to_vec();
    if let Some(current) = env::var_os("PATH") {
        paths.extend(env::split_paths(&current));
    }
    let mut unique = Vec::with_capacity(paths.len());
    let mut seen = std::collections::HashSet::new();
    for path in paths {
        if seen.insert(path.clone()) {
            unique.push(path);
        }
    }
    if let Ok(joined) = env::join_paths(unique) {
        // SAFETY: process-local PATH update before native OCR/CUDA libraries load.
        unsafe { env::set_var("PATH", joined) };
    }
}

#[cfg(test)]
mod tests {
    use super::runtime_bin_priority;
    use std::path::Path;

    #[test]
    fn cuda_runtime_bins_rank_above_cpu() {
        assert!(
            runtime_bin_priority(Path::new(
                r"C:\app\thirdparty\uocr-runtime\windows-x86_64-cuda13\bin"
            )) > runtime_bin_priority(Path::new(
                r"C:\app\thirdparty\uocr-runtime\windows-x86_64-cpu\bin"
            ))
        );
    }
}
