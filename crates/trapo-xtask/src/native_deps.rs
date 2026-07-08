use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use serde::Serialize;

use crate::{
    archive::{download_if_needed, extract_archive},
    manifest::{self, NativeDependencyTarget, OnnxRuntimeManifest},
};

#[derive(Debug, Serialize)]
struct PreparedNativeDependency {
    archive: String,
    archive_root: String,
    asset: String,
    include_dir: String,
    library: String,
    library_dir: String,
    notice_files: Vec<String>,
    onnxruntime_version: String,
    platform: String,
    release_tag: String,
    root: String,
    runtime_libraries: Vec<String>,
    sha256: String,
}

pub(crate) fn prepare_native_deps(args: &[String]) -> Result<()> {
    let repo_root = repo_root_from_args(args)?;
    let platform =
        arg_value(args, "--platform").ok_or_else(|| anyhow!("--platform is required"))?;
    let deps_manifest = manifest::load_manifest(&repo_root)?;
    manifest::check_onnx_submodule(&repo_root, &deps_manifest)?;
    let target = deps_manifest
        .onnxruntime
        .targets
        .get(platform)
        .ok_or_else(|| anyhow!("no native dependency target for platform {platform}"))?;

    let deps_root = repo_root.join(".deps").join("native").join(platform);
    let downloads = repo_root.join(".deps").join("downloads");
    let archive = downloads.join(&target.asset);
    fs::create_dir_all(&downloads)?;
    download_if_needed(&deps_manifest.onnxruntime.base_url, target, &archive)?;

    let extract_root = deps_root.join("onnxruntime");
    if extract_root.exists() {
        fs::remove_dir_all(&extract_root).with_context(|| {
            format!(
                "failed to remove stale native dependency directory {}",
                extract_root.display()
            )
        })?;
    }
    fs::create_dir_all(&extract_root)?;
    extract_archive(target, &archive, &extract_root)?;

    let root = extract_root.join(&target.archive_root);
    let receipt = prepared_receipt(
        platform,
        &deps_manifest.onnxruntime,
        target,
        &archive,
        &root,
    );
    validate_prepared_files(&receipt)?;
    let receipt_path = deps_root.join("onnxruntime.json");
    fs::write(
        &receipt_path,
        serde_json::to_string_pretty(&receipt)? + "\n",
    )?;
    write_json_line(&receipt)
}

pub(crate) fn repo_root_from_args(args: &[String]) -> Result<PathBuf> {
    arg_value(args, "--repo-root")
        .map_or_else(
            || env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            PathBuf::from,
        )
        .canonicalize()
        .context("failed to resolve repository root")
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|window| (window[0] == name).then_some(window[1].as_str()))
}

fn prepared_receipt(
    platform: &str,
    runtime: &OnnxRuntimeManifest,
    target: &NativeDependencyTarget,
    archive: &Path,
    root: &Path,
) -> PreparedNativeDependency {
    PreparedNativeDependency {
        archive: path_string(archive),
        archive_root: target.archive_root.clone(),
        asset: target.asset.clone(),
        include_dir: path_string(&root.join(&target.include_dir)),
        library: path_string(&root.join(&target.library)),
        library_dir: path_string(&root.join(&target.library_dir)),
        notice_files: target
            .notice_files
            .iter()
            .map(|path| path_string(&root.join(path)))
            .collect(),
        onnxruntime_version: runtime.version.clone(),
        platform: platform.to_string(),
        release_tag: runtime.release_tag.clone(),
        root: path_string(root),
        runtime_libraries: target
            .runtime_libraries
            .iter()
            .map(|path| path_string(&root.join(path)))
            .collect(),
        sha256: target.sha256.clone(),
    }
}

fn path_string(path: &Path) -> String {
    let value = path.display().to_string();
    if cfg!(windows) {
        value
            .strip_prefix("\\\\?\\")
            .unwrap_or(value.as_str())
            .to_string()
    } else {
        value
    }
}

fn validate_prepared_files(receipt: &PreparedNativeDependency) -> Result<()> {
    for path in [&receipt.include_dir, &receipt.library] {
        if !Path::new(path).exists() {
            anyhow::bail!("prepared native dependency path is missing: {path}");
        }
    }
    for path in receipt
        .runtime_libraries
        .iter()
        .chain(receipt.notice_files.iter())
    {
        if !Path::new(path).is_file() {
            anyhow::bail!("prepared native dependency file is missing: {path}");
        }
    }
    Ok(())
}

fn write_json_line(value: &PreparedNativeDependency) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn test_target(asset: &str, archive_type: &str, sha256: &str) -> NativeDependencyTarget {
    NativeDependencyTarget {
        archive_root: "onnxruntime-test".to_string(),
        archive_type: archive_type.to_string(),
        asset: asset.to_string(),
        include_dir: "include".to_string(),
        library: "lib/onnxruntime.lib".to_string(),
        library_dir: "lib".to_string(),
        notice_files: vec!["LICENSE".to_string(), "ThirdPartyNotices.txt".to_string()],
        runtime_libraries: vec!["lib/onnxruntime.dll".to_string()],
        sha256: sha256.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::{Result, anyhow};

    use super::*;
    use crate::archive::sha256_file;

    #[test]
    fn prepared_receipt_validates_notice_files() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("onnxruntime-test");
        fs::create_dir_all(root.join("include"))?;
        fs::create_dir_all(root.join("lib"))?;
        fs::write(root.join("lib").join("onnxruntime.dll"), b"dll")?;
        fs::write(root.join("lib").join("onnxruntime.lib"), b"lib")?;
        fs::write(root.join("LICENSE"), b"license")?;
        fs::write(root.join("ThirdPartyNotices.txt"), b"notice")?;
        let archive = temp.path().join("native.zip");
        fs::write(&archive, b"archive")?;
        let target = test_target("native.zip", "zip", &sha256_file(&archive)?);
        let runtime = OnnxRuntimeManifest {
            base_url: "https://example.invalid".to_string(),
            release_tag: "v1.27.0".to_string(),
            targets: BTreeMap::default(),
            version: "1.27.0".to_string(),
        };

        let receipt = prepared_receipt("windows-x86_64-cpu", &runtime, &target, &archive, &root);

        validate_prepared_files(&receipt)?;
        assert!(
            receipt
                .notice_files
                .iter()
                .any(|path| path.ends_with("ThirdPartyNotices.txt"))
        );
        Ok(())
    }

    #[test]
    fn manifest_has_windows_cuda13_target() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| anyhow!("crate path has no repo root"))?
            .to_path_buf();
        let deps_manifest = manifest::load_manifest(&repo_root)?;
        let target = deps_manifest
            .onnxruntime
            .targets
            .get("windows-x86_64-cuda13")
            .ok_or_else(|| anyhow!("windows cuda13 ORT target missing"))?;
        assert!(
            target
                .runtime_libraries
                .iter()
                .any(|path| path.contains("cuda"))
        );
        assert_eq!(deps_manifest.onnx.required_tag, "v1.21.0");
        Ok(())
    }
}
