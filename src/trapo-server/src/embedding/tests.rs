use serde_json::json;

use super::*;
use crate::catalog::embedding_model_catalog;

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

    let profile = profile_from_model_row(temp.path(), temp.path(), &row, None)?;

    assert_eq!(profile.dimension, 768);
    assert_eq!(profile.context_tokens, 8192);
    assert_eq!(profile.n_gpu_layers, 12);
    assert_eq!(profile.n_batch, 1024);
    assert_eq!(profile.n_ubatch, 512);
    assert!(matches!(profile.pooling, PoolingType::Mean));
    Ok(())
}

#[test]
fn resolves_selected_packaged_runtime_before_cpu_fallback() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let cpu_path = install_runtime_library(temp.path(), "windows-x86_64-cpu")?;
    let cuda_path = install_runtime_library(temp.path(), "windows-x86_64-cuda13")?;

    let expected_cuda = cuda_path.canonicalize()?;
    assert_eq!(
        resolve_llama_library(temp.path(), Some("windows-x86_64-cuda13")).as_deref(),
        Some(expected_cuda.as_path())
    );

    let expected_cpu = cpu_path.canonicalize()?;
    assert_eq!(
        resolve_llama_library(temp.path(), None).as_deref(),
        Some(expected_cpu.as_path())
    );
    Ok(())
}

#[test]
fn l2_normalization_handles_zero_vector() {
    let mut vector = vec![0.0, 0.0, 0.0];
    normalize_l2(&mut vector);
    assert_eq!(vector, vec![0.0, 0.0, 0.0]);
}

#[test]
fn catalog_embedding_profiles_have_safe_encoder_batch_ceiling() -> Result<()> {
    for entry in embedding_model_catalog() {
        let params: serde_json::Value = serde_json::from_str(entry.llama_params_json)?;
        let n_batch = json_u32(&params, "n_batch").unwrap_or(512);
        let n_ubatch = json_u32(&params, "n_ubatch").unwrap_or(512);
        let context_tokens = entry.context_tokens.unwrap_or_default();
        let profile = LlamaEmbeddingProfile {
            model_id: entry.model_id.to_string(),
            model_path: PathBuf::from(entry.model_file),
            library_path: PathBuf::from(llama_library_name()),
            dimension: entry.embedding_dimension.unwrap_or_default(),
            context_tokens,
            pooling: PoolingType::from_catalog(entry.pooling)?,
            normalize: entry.normalize_embeddings,
            query_prefix: entry.query_prefix.to_string(),
            document_prefix: entry.document_prefix.to_string(),
            n_gpu_layers: json_i32(&params, "n_gpu_layers").unwrap_or(99),
            n_batch,
            n_ubatch,
        };

        assert!(
            profile.effective_batch_tokens() >= 512,
            "{} has an unsafe llama.cpp encoder batch ceiling: {:?}",
            entry.model_id,
            entry.llama_params_json
        );
    }
    Ok(())
}

fn install_runtime_library(root: &Path, runtime_id: &str) -> Result<PathBuf> {
    let runtime_bin = root
        .join("thirdparty")
        .join("uocr-runtime")
        .join(runtime_id)
        .join("bin");
    std::fs::create_dir_all(&runtime_bin)?;
    let library_path = runtime_bin.join(llama_library_name());
    std::fs::write(&library_path, "")?;
    Ok(library_path)
}
