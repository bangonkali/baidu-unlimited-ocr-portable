use std::{collections::BTreeMap, fs, path::Path, process::Command};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct NativeDepsManifest {
    pub(crate) onnx: OnnxManifest,
    pub(crate) onnxruntime: OnnxRuntimeManifest,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OnnxManifest {
    pub(crate) required_tag: String,
    pub(crate) required_commit: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OnnxRuntimeManifest {
    pub(crate) base_url: String,
    pub(crate) release_tag: String,
    pub(crate) targets: BTreeMap<String, NativeDependencyTarget>,
    pub(crate) version: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct NativeDependencyTarget {
    pub(crate) archive_root: String,
    pub(crate) archive_type: String,
    pub(crate) asset: String,
    pub(crate) include_dir: String,
    pub(crate) library: String,
    pub(crate) library_dir: String,
    pub(crate) notice_files: Vec<String>,
    pub(crate) runtime_libraries: Vec<String>,
    pub(crate) sha256: String,
}

pub(crate) fn load_manifest(repo_root: &Path) -> Result<NativeDepsManifest> {
    let path = repo_root.join("runtime").join("native-deps.json");
    let text = fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read native dependency manifest {}",
            path.display()
        )
    })?;
    serde_json::from_str(&text).context("failed to parse native dependency manifest")
}

pub(crate) fn check_onnx_submodule(repo_root: &Path, manifest: &NativeDepsManifest) -> Result<()> {
    let onnx_root = repo_root.join("thirdparty").join("onnx");
    let actual = git_output(&onnx_root, &["rev-parse", "HEAD"]).with_context(|| {
        format!(
            "failed to inspect ONNX submodule at {}",
            onnx_root.display()
        )
    })?;
    if actual != manifest.onnx.required_commit {
        bail!(
            "thirdparty/onnx must be {} ({}) for ORT {}; found {actual}",
            manifest.onnx.required_tag,
            manifest.onnx.required_commit,
            manifest.onnxruntime.version
        );
    }
    Ok(())
}

fn git_output(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(cwd).output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
