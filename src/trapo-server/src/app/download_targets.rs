#[derive(Debug, Clone)]
struct DownloadTarget {
    download_key: String,
    owner_kind: String,
    owner_id: String,
    file_id: String,
    file_name: String,
    source_url: String,
    target_path: PathBuf,
    total_bytes: u64,
}

fn model_download_targets(
    model_dir: &Path,
    entry: &crate::catalog::ModelCatalogEntry,
) -> Vec<DownloadTarget> {
    let mut files = vec![("model", entry.model_file, entry.model_size_bytes)];
    if let Some(mmproj_file) = entry.mmproj_file {
        files.push(("mmproj", mmproj_file, entry.mmproj_size_bytes));
    }
    files
        .into_iter()
    .map(|(file_id, file_name, total_bytes)| DownloadTarget {
        download_key: model_download_key(entry.model_id, file_id),
        owner_kind: "model".to_string(),
        owner_id: entry.model_id.to_string(),
        file_id: file_id.to_string(),
        file_name: file_name.to_string(),
        source_url: hf_resolve_url(entry.repo_id, entry.revision, file_name),
        target_path: model_dir.join(file_name),
        total_bytes,
    })
    .collect()
}

fn model_download_key(model_id: &str, file_id: &str) -> String {
    format!("model:{model_id}:{file_id}")
}

fn hf_resolve_url(repo_id: &str, revision: &str, file_name: &str) -> String {
    format!(
        "https://huggingface.co/{repo_id}/resolve/{revision}/{file_name}"
    )
}

fn file_is_present(path: &Path) -> bool {
    path.is_file()
        && path
            .metadata()
            .map_or(0, |metadata| metadata.len())
            > 0
}

fn is_active_download_status(status: &str) -> bool {
    matches!(status, "queued" | "downloading" | "cancelling")
}
