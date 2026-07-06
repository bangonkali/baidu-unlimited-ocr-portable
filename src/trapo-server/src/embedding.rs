#![allow(unsafe_code)]

mod ffi;
mod session;
mod split;
#[cfg(test)]
mod tests;
mod worker;

use std::{
    collections::BTreeSet,
    ffi::OsStr,
    os::raw::c_int,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use session::LlamaEmbeddingSession;

use crate::{
    error::{AppError, Result},
    storage::RagEmbeddingModelRow,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum EmbeddingPurpose {
    Document,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LlamaEmbeddingProfile {
    pub(crate) model_id: String,
    pub(crate) model_path: PathBuf,
    pub(crate) library_path: PathBuf,
    pub(crate) dimension: u32,
    pub(crate) context_tokens: u32,
    pub(crate) pooling: PoolingType,
    pub(crate) normalize: bool,
    pub(crate) query_prefix: String,
    pub(crate) document_prefix: String,
    pub(crate) n_gpu_layers: i32,
    pub(crate) n_batch: u32,
    pub(crate) n_ubatch: u32,
}

impl LlamaEmbeddingProfile {
    pub(crate) fn effective_batch_tokens(&self) -> u32 {
        self.context_tokens
            .min(self.n_batch.max(self.n_ubatch))
            .max(1)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum PoolingType {
    Mean,
    Cls,
    Last,
}

pub(crate) fn profile_from_model_row(
    app_root: &Path,
    model_dir: &Path,
    row: &RagEmbeddingModelRow,
    preferred_runtime_id: Option<&str>,
) -> Result<LlamaEmbeddingProfile> {
    let params = &row.llama_params;
    Ok(LlamaEmbeddingProfile {
        model_id: row.model_id.clone(),
        model_path: resolve_model_file(model_dir, &row.filename)?,
        library_path: resolve_llama_library(app_root, preferred_runtime_id).ok_or_else(|| {
            AppError::BadRequest("llama.cpp runtime library was not found".to_string())
        })?,
        dimension: row.dimension,
        context_tokens: row.context_tokens,
        pooling: PoolingType::from_catalog(row.pooling.as_str())?,
        normalize: row.normalize,
        query_prefix: row.query_prefix.clone(),
        document_prefix: row.document_prefix.clone(),
        n_gpu_layers: json_i32(params, "n_gpu_layers").unwrap_or(99),
        n_batch: json_u32(params, "n_batch").unwrap_or(512),
        n_ubatch: json_u32(params, "n_ubatch").unwrap_or(256),
    })
}

pub(crate) fn resolve_llama_library(
    app_root: &Path,
    preferred_runtime_id: Option<&str>,
) -> Option<PathBuf> {
    llama_library_candidates(app_root, preferred_runtime_id)
        .into_iter()
        .find_map(|path| path.is_file().then(|| path.canonicalize().ok()).flatten())
}

/// Validates that a local llama.cpp runtime library can be loaded for embeddings.
///
/// # Errors
///
/// Returns an error when the dynamic library cannot be loaded or when required
/// embedding symbols are missing.
pub fn validate_llama_library(path: &Path) -> Result<()> {
    ffi::validate_llama_library(path)
}

pub(crate) async fn generate_embeddings(
    profile: LlamaEmbeddingProfile,
    purpose: EmbeddingPurpose,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>> {
    worker::generate_embeddings_with_worker(profile, purpose, texts).await
}

/// Runs an embedding worker request in the current process.
///
/// # Errors
///
/// Returns an error when request/response JSON cannot be read or written, or
/// when llama.cpp embedding generation fails.
pub fn run_embedding_worker(request_path: &Path, response_path: &Path) -> Result<()> {
    worker::run_embedding_worker(request_path, response_path)
}

pub(super) fn generate_embeddings_in_process(
    profile: &LlamaEmbeddingProfile,
    purpose: EmbeddingPurpose,
    texts: &[String],
) -> Result<Vec<Vec<f32>>> {
    let mut session = LlamaEmbeddingSession::open(profile)?;
    texts
        .iter()
        .map(|text| session.embed_text(text, purpose))
        .collect()
}

impl PoolingType {
    fn from_catalog(value: &str) -> Result<Self> {
        match value {
            "mean" => Ok(Self::Mean),
            "cls" => Ok(Self::Cls),
            "last" => Ok(Self::Last),
            _ => Err(AppError::BadRequest(format!(
                "unsupported llama.cpp embedding pooling type: {value}"
            ))),
        }
    }

    pub(super) const fn ffi_value(self) -> c_int {
        match self {
            Self::Mean => 1,
            Self::Cls => 2,
            Self::Last => 3,
        }
    }
}

fn normalize_l2(vector: &mut [f32]) {
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 && norm.is_finite() {
        for value in vector {
            *value /= norm;
        }
    }
}

fn json_u32(value: &Value, key: &str) -> Option<u32> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|raw| u32::try_from(raw).ok())
}

fn json_i32(value: &Value, key: &str) -> Option<i32> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|raw| i32::try_from(raw).ok())
}

fn resolve_model_file(model_dir: &Path, filename: &str) -> Result<PathBuf> {
    let file_name = Path::new(filename);
    if file_name.file_name() != Some(OsStr::new(filename)) {
        return Err(AppError::BadRequest(format!(
            "embedding model filename must not contain path separators: {filename}"
        )));
    }
    let model_root = model_dir.canonicalize().map_err(|error| {
        AppError::BadRequest(format!(
            "embedding model directory is not available: {error}"
        ))
    })?;
    let model_path = model_root.join(file_name);
    let canonical = model_path.canonicalize().map_err(|error| {
        AppError::BadRequest(format!(
            "embedding model file is missing: {} ({error})",
            model_path.display()
        ))
    })?;
    if !canonical.starts_with(&model_root) {
        return Err(AppError::BadRequest(
            "embedding model file must stay inside the configured model directory".to_string(),
        ));
    }
    Ok(canonical)
}

fn llama_library_candidates(app_root: &Path, preferred_runtime_id: Option<&str>) -> Vec<PathBuf> {
    let library_name = llama_library_name();
    let mut candidates =
        packaged_runtime_llama_candidates(app_root, library_name, preferred_runtime_id);
    candidates.extend([
        app_root
            .join("thirdparty")
            .join("llama.cpp")
            .join("build")
            .join("bin")
            .join("Release")
            .join(library_name),
        app_root
            .join("thirdparty")
            .join("llama.cpp")
            .join("build")
            .join("bin")
            .join(library_name),
        app_root
            .join("thirdparty")
            .join("llama")
            .join("bin")
            .join(library_name),
    ]);
    candidates.push(app_root.join(library_name));
    candidates
}

fn packaged_runtime_llama_candidates(
    app_root: &Path,
    library_name: &str,
    preferred_runtime_id: Option<&str>,
) -> Vec<PathBuf> {
    let runtime_root = app_root.join("thirdparty").join("uocr-runtime");
    let Ok(entries) = std::fs::read_dir(&runtime_root) else {
        return Vec::new();
    };
    let mut runtimes: Vec<String> = entries
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    runtimes.sort_by_key(|runtime_id| runtime_rank(runtime_id, preferred_runtime_id));
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    for runtime_id in runtimes {
        let candidate = runtime_root.join(runtime_id).join("bin").join(library_name);
        if seen.insert(candidate.clone()) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn runtime_rank(runtime_id: &str, preferred_runtime_id: Option<&str>) -> (u8, String) {
    if Some(runtime_id) == preferred_runtime_id {
        (0, runtime_id.to_string())
    } else if runtime_id.ends_with("-cpu") {
        (1, runtime_id.to_string())
    } else {
        (2, runtime_id.to_string())
    }
}

const fn llama_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "llama.dll"
    } else if cfg!(target_os = "macos") {
        "libllama.dylib"
    } else {
        "libllama.so"
    }
}
