//! Process-level logging and worker isolation integration tests.

use std::{fs, process::Command};

use serde_json::json;

#[test]
fn process_logging_captures_stdio_and_panic() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let log_dir = temp.path().join("logs");
    let server = server_binary();

    let status = Command::new(&server)
        .arg("--self-test-log-stdio")
        .arg(&log_dir)
        .status()?;
    assert!(status.success());

    let log_path = log_dir.join("trapo-server.log");
    let log = fs::read_to_string(&log_path)?;
    assert!(log.contains("stdio capture initialized"));
    assert!(log.contains("self-test stdout marker"));
    assert!(log.contains("self-test stderr marker"));

    let status = Command::new(&server)
        .arg("--self-test-log-child-stderr")
        .arg(&log_dir)
        .status()?;
    assert!(status.success());

    let log = fs::read_to_string(&log_path)?;
    assert!(log.contains("self-test child stderr marker"));

    let status = Command::new(&server)
        .arg("--self-test-log-panic")
        .arg(&log_dir)
        .status()?;
    assert!(!status.success());

    let log = fs::read_to_string(log_path)?;
    assert!(log.contains("panic"));
    assert!(log.contains("self-test panic marker"));
    Ok(())
}

#[test]
fn embedding_worker_failure_stays_in_child_process() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let request_path = temp.path().join("embedding-request.json");
    let response_path = temp.path().join("embedding-response.json");
    fs::write(
        &request_path,
        serde_json::to_vec(&json!({
            "profile": {
                "model_id": "test-model",
                "model_path": temp.path().join("missing-model.gguf"),
                "library_path": temp.path().join("missing-llama.dll"),
                "dimension": 8,
                "context_tokens": 128,
                "pooling": "Mean",
                "normalize": true,
                "query_prefix": "",
                "document_prefix": "",
                "n_gpu_layers": 0,
                "n_batch": 32,
                "n_ubatch": 32
            },
            "purpose": "Document",
            "texts": ["hello"]
        }))?,
    )?;

    let status = Command::new(server_binary())
        .arg("--embedding-worker")
        .arg(&request_path)
        .arg(&response_path)
        .status()?;

    assert!(!status.success());
    assert!(!response_path.exists());
    Ok(())
}

fn server_binary() -> String {
    env!("CARGO_BIN_EXE_trapo-server").to_string()
}
