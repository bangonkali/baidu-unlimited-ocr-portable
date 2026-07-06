#![allow(unsafe_code)]

mod ffi;
mod session;

use std::{
    ffi::OsStr,
    os::raw::c_int,
    path::{Path, PathBuf},
};

use serde_json::Value;
use session::LlamaEmbeddingSession;

use crate::{
    error::{AppError, Result},
    storage::RagEmbeddingModelRow,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum EmbeddingPurpose {
    Document,
    Query,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum PoolingType {
    Mean,
    Cls,
    Last,
}

pub(crate) fn profile_from_model_row(
    app_root: &Path,
    model_dir: &Path,
    row: &RagEmbeddingModelRow,
) -> Result<LlamaEmbeddingProfile> {
    let params = &row.llama_params;
    Ok(LlamaEmbeddingProfile {
        model_id: row.model_id.clone(),
        model_path: resolve_model_file(model_dir, &row.filename)?,
        library_path: resolve_llama_library(app_root).ok_or_else(|| {
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

pub(crate) fn resolve_llama_library(app_root: &Path) -> Option<PathBuf> {
    llama_library_candidates(app_root)
        .into_iter()
        .find_map(|path| path.is_file().then(|| path.canonicalize().ok()).flatten())
}

pub(crate) async fn generate_embeddings(
    profile: LlamaEmbeddingProfile,
    purpose: EmbeddingPurpose,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>> {
    tokio::task::spawn_blocking(move || {
        let mut session = LlamaEmbeddingSession::open(&profile)?;
        texts
            .iter()
            .map(|text| session.embed_text(text, purpose))
            .collect()
    })
    .await
    .map_err(|error| AppError::Internal(format!("embedding worker failed: {error}")))?
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

fn llama_library_candidates(app_root: &Path) -> Vec<PathBuf> {
    let library_name = llama_library_name();
    vec![
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
        app_root.join(library_name),
    ]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn profile_uses_catalog_tuned_parameters() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let row = RagEmbeddingModelRow {
            model_id: "nomic".to_string(),
            display_name: "Nomic".to_string(),
            provider: "Nomic".to_string(),
            repo_id: "repo".to_string(),
            filename: "model.gguf".to_string(),
            revision: "main".to_string(),
            routing_origin: "embedding".to_string(),
            model_family: "MRL".to_string(),
            dimension: 768,
            context_tokens: 8192,
            pooling: "mean".to_string(),
            normalize: true,
            query_prefix: "search_query: ".to_string(),
            document_prefix: "search_document: ".to_string(),
            llama_params: json!({"n_gpu_layers": 12, "n_batch": 1024, "n_ubatch": 512}),
            recommended_vram_gb: 4.0,
            active: true,
        };
        std::fs::create_dir_all(
            temp.path()
                .join("thirdparty")
                .join("llama.cpp")
                .join("build")
                .join("bin")
                .join("Release"),
        )?;
        std::fs::write(
            temp.path()
                .join("thirdparty")
                .join("llama.cpp")
                .join("build")
                .join("bin")
                .join("Release")
                .join(llama_library_name()),
            "",
        )?;
        std::fs::write(temp.path().join("model.gguf"), "")?;
        let profile = profile_from_model_row(temp.path(), temp.path(), &row)?;
        assert_eq!(profile.dimension, 768);
        assert_eq!(profile.context_tokens, 8192);
        assert_eq!(profile.n_gpu_layers, 12);
        assert_eq!(profile.n_batch, 1024);
        assert_eq!(profile.n_ubatch, 512);
        assert!(matches!(profile.pooling, PoolingType::Mean));
        Ok(())
    }

    #[test]
    fn l2_normalization_handles_zero_vector() {
        let mut vector = vec![0.0, 0.0, 0.0];
        normalize_l2(&mut vector);
        assert_eq!(vector, vec![0.0, 0.0, 0.0]);
    }
}
