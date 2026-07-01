use std::{
    fmt::Write as _,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use walkdir::WalkDir;

use crate::error::{AppError, Result};

pub const SUPPORTED_INPUTS: &[&str] = &["pdf", "png", "jpg", "jpeg", "bmp", "tif", "tiff", "webp"];

#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub size_bytes: u64,
    pub modified_at: Option<SystemTime>,
}

pub fn is_supported_document(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|extension| {
            SUPPORTED_INPUTS
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
        .unwrap_or(false)
}

pub fn validate_trusted_root(root: &Path) -> Result<PathBuf> {
    let metadata = std::fs::symlink_metadata(root)
        .map_err(|_| AppError::BadRequest("folder does not exist".to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::BadRequest(
            "folder symlinks are not accepted".to_string(),
        ));
    }
    if !metadata.is_dir() {
        return Err(AppError::BadRequest("path is not a folder".to_string()));
    }
    std::fs::canonicalize(root)
        .map_err(|_| AppError::BadRequest("folder cannot be resolved".to_string()))
}

pub fn discover_supported_files(root: &Path) -> Result<Vec<DiscoveredFile>> {
    let safe_root = validate_trusted_root(root)?;
    let mut files = Vec::new();
    for entry in WalkDir::new(&safe_root).follow_links(false).into_iter() {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() || !is_supported_document(entry.path()) {
            continue;
        }
        let Ok(absolute_path) = std::fs::canonicalize(entry.path()) else {
            continue;
        };
        let relative_path = absolute_path
            .strip_prefix(&safe_root)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| absolute_path.clone());
        files.push(DiscoveredFile {
            absolute_path,
            relative_path,
            size_bytes: metadata.len(),
            modified_at: metadata.modified().ok(),
        });
    }
    files.sort_by(|left, right| {
        generic_path(&left.relative_path).cmp(&generic_path(&right.relative_path))
    });
    Ok(files)
}

pub fn stable_hash(file: &DiscoveredFile) -> String {
    let mut hash = 1_469_598_103_934_665_603_u64;
    mix_fnv1a(&mut hash, &generic_path(&file.absolute_path));
    mix_fnv1a(&mut hash, &file.size_bytes.to_string());
    format!("{hash:016x}")
}

pub fn generic_path(path: &Path) -> String {
    let mut out = String::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push_str(&prefix.as_os_str().to_string_lossy()),
            Component::RootDir => {
                if !out.ends_with('/') {
                    out.push('/');
                }
            }
            Component::CurDir => push_component(&mut out, "."),
            Component::ParentDir => push_component(&mut out, ".."),
            Component::Normal(value) => push_component(&mut out, &value.to_string_lossy()),
        }
    }
    out
}

pub fn region_hash_key(parts: impl IntoIterator<Item = String>) -> String {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for part in parts {
        mix_fnv1a(&mut hash, &part);
    }
    let mut out = String::from("reg_");
    let _ = write!(out, "{hash:016x}");
    out
}

fn mix_fnv1a(hash: &mut u64, value: &str) {
    for byte in value.as_bytes() {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(1_099_511_628_211);
    }
}

fn push_component(out: &mut String, value: &str) {
    if !out.is_empty() && !out.ends_with('/') {
        out.push('/');
    }
    out.push_str(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_expected_extensions_case_insensitively() {
        assert!(is_supported_document(Path::new("a.PDF")));
        assert!(is_supported_document(Path::new("a.webp")));
        assert!(!is_supported_document(Path::new("a.txt")));
    }
}
