use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use serde::{Deserialize, Serialize};

use crate::{
    embedding::{EmbeddingPurpose, LlamaEmbeddingProfile, generate_embeddings_in_process},
    error::{AppError, Result},
    ids::new_persistence_id,
};

pub(super) async fn generate_embeddings_with_worker(
    profile: LlamaEmbeddingProfile,
    purpose: EmbeddingPurpose,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>> {
    tokio::task::spawn_blocking(move || run_worker_batch(profile, purpose, texts))
        .await
        .map_err(|error| AppError::Internal(format!("embedding worker join failed: {error}")))?
}

// skylos: ignore[unused_functions] invoked through embedding/lib exports by the hidden --embedding-worker CLI.
pub(super) fn run_embedding_worker(request_path: &Path, response_path: &Path) -> Result<()> {
    let request_path = canonical_existing_file(request_path, "embedding worker request")?;
    let response_path = canonical_output_file(response_path, "embedding worker response")?;
    let request: EmbeddingWorkerRequest = serde_json::from_slice(&std::fs::read(&request_path)?)?; // skylos: ignore[SKY-D215] worker request path is canonicalized before this internal IPC read.
    let embeddings =
        generate_embeddings_in_process(&request.profile, request.purpose, &request.texts)?;
    let response = EmbeddingWorkerResponse { embeddings };
    std::fs::write(&response_path, serde_json::to_vec(&response)?)?; // skylos: ignore[SKY-D215] worker response parent is canonicalized before this internal IPC write.
    Ok(())
}

fn run_worker_batch(
    profile: LlamaEmbeddingProfile,
    purpose: EmbeddingPurpose,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>> {
    let worker_paths = WorkerPaths::new()?;
    std::fs::write(
        &worker_paths.request,
        serde_json::to_vec(&EmbeddingWorkerRequest {
            profile,
            purpose,
            texts,
        })?,
    )?;
    let status = run_worker_process(&worker_paths.request, &worker_paths.response);
    let response = match status {
        Ok(status) if status.success() => read_worker_response(&worker_paths.response),
        Ok(status) => Err(worker_exit_error(status)),
        Err(error) => Err(AppError::Internal(format!(
            "failed to start embedding worker: {error}"
        ))),
    };
    worker_paths.cleanup();
    response
}

fn run_worker_process(request_path: &Path, response_path: &Path) -> std::io::Result<ExitStatus> {
    let mut command = Command::new(env::current_exe()?);
    command
        .arg("--embedding-worker")
        .arg(request_path)
        .arg(response_path);
    hide_worker_window(&mut command);
    command.status()
}

#[cfg(windows)]
fn hide_worker_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
const fn hide_worker_window(_command: &mut Command) {}

fn read_worker_response(response_path: &Path) -> Result<Vec<Vec<f32>>> {
    let response_bytes = std::fs::read(response_path) // skylos: ignore[SKY-D215] response path is generated under the canonical temp worker directory.
        .map_err(|error| {
            AppError::Internal(format!(
                "embedding worker did not write a response at {}: {error}",
                response_path.display()
            ))
        })?;
    let response: EmbeddingWorkerResponse = serde_json::from_slice(&response_bytes)?;
    Ok(response.embeddings)
}

fn worker_exit_error(status: ExitStatus) -> AppError {
    AppError::Internal(format!(
        "embedding worker exited before producing vectors with status {status}; see trapo-server.log for native llama.cpp output"
    ))
}

#[derive(Debug)]
struct WorkerPaths {
    request: PathBuf,
    response: PathBuf,
}

impl WorkerPaths {
    fn new() -> Result<Self> {
        let worker_dir = env::temp_dir().join("trapo-embedding-workers");
        std::fs::create_dir_all(&worker_dir)?;
        let worker_dir = worker_dir.canonicalize()?;
        let worker_id = new_persistence_id();
        Ok(Self {
            request: worker_dir.join(format!("{worker_id}.request.json")),
            response: worker_dir.join(format!("{worker_id}.response.json")),
        })
    }

    fn cleanup(&self) {
        let _ = std::fs::remove_file(&self.request);
        let _ = std::fs::remove_file(&self.response);
    }
}

fn canonical_existing_file(path: &Path, description: &str) -> Result<PathBuf> {
    path.canonicalize().map_err(|error| {
        AppError::BadRequest(format!(
            "{description} path is not readable at {}: {error}",
            path.display()
        ))
    })
}

fn canonical_output_file(path: &Path, description: &str) -> Result<PathBuf> {
    let Some(parent) = path.parent() else {
        return Err(AppError::BadRequest(format!(
            "{description} path must include a parent directory"
        )));
    };
    let Some(file_name) = path.file_name() else {
        return Err(AppError::BadRequest(format!(
            "{description} path must include a file name"
        )));
    };
    let parent = parent.canonicalize().map_err(|error| {
        AppError::BadRequest(format!(
            "{description} parent is not readable at {}: {error}",
            parent.display()
        ))
    })?;
    Ok(parent.join(file_name))
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingWorkerRequest {
    profile: LlamaEmbeddingProfile,
    purpose: EmbeddingPurpose,
    texts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingWorkerResponse {
    embeddings: Vec<Vec<f32>>,
}
