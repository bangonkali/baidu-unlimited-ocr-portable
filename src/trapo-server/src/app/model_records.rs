fn model_record(
    model_dir: &Path,
    state: &WorkbenchState,
    entry: &crate::catalog::ModelCatalogEntry,
) -> ModelAssetRecord {
    let files = model_files(model_dir, state, entry);
    let status = model_status_from_files(&files);
    let total_required = entry.model_size_bytes + entry.mmproj_size_bytes;
    let total_file_count = usize_to_u32_saturating(files.len());
    let downloaded_bytes = files.iter().map(|file| file.downloaded_bytes).sum::<u64>();
    let percent = download_percent(downloaded_bytes, total_required);
    let active_downloads = model_downloads(state, entry.model_id);
    let bytes_per_second = active_downloads
        .iter()
        .map(|download| download_rate(download))
        .sum::<f64>();
    ModelAssetRecord {
        model_id: entry.model_id.to_string(),
        display_name: entry.display_name.to_string(),
        model_kind: entry.model_kind.to_string(),
        routing_origin: entry.routing_origin.to_string(),
        status: status.clone(),
        repo_id: entry.repo_id.to_string(),
        revision: entry.revision.to_string(),
        local_path: (status == "downloaded")
            .then(|| model_dir.join(entry.model_file).to_string_lossy().to_string()),
        size_bytes: Some(entry.model_size_bytes),
        error: files.iter().find_map(|file| file.error.clone()),
        error_kind: files.iter().find_map(|file| file.error_kind.clone()),
        model_file: entry.model_file.to_string(),
        mmproj_file: entry.mmproj_file.unwrap_or("").to_string(),
        current_file: active_downloads
            .iter()
            .find(|download| download.status == "downloading")
            .map(|download| download.file_name.clone()),
        status_message: Some(model_status_message(&status).to_string()),
        downloaded_bytes,
        total_bytes: Some(total_required),
        overall_downloaded_bytes: downloaded_bytes,
        overall_total_bytes: Some(total_required),
        overall_percent: if status == "downloaded" { 100.0 } else { percent },
        bytes_per_second,
        eta_seconds: eta_seconds(total_required.saturating_sub(downloaded_bytes), bytes_per_second),
        auth_available: hf_token().is_some(),
        auth_source: hf_token().map(|_| "environment".to_string()),
        last_event_at: active_downloads
            .iter()
            .filter_map(|download| download.last_event_at.clone())
            .max(),
        files,
        quantization: entry.quantization.to_string(),
        bits: entry.bits,
        quality: entry.quality.to_string(),
        hardware_tier: entry.hardware_tier.to_string(),
        notes: entry.notes.to_string(),
        recommended: entry.recommended,
        selected: state.selected_model_id == entry.model_id,
        provider_name: entry.provider_label.to_string(),
        embedding_dimension: entry.embedding_dimension,
        context_tokens: entry.context_tokens,
        pooling: (!entry.pooling.is_empty()).then(|| entry.pooling.to_string()),
        normalize_embeddings: entry.normalize_embeddings,
        query_prefix: (!entry.query_prefix.is_empty()).then(|| entry.query_prefix.to_string()),
        document_prefix: (!entry.document_prefix.is_empty()).then(|| entry.document_prefix.to_string()),
        recommended_vram_gb: (entry.recommended_vram_gb > 0.0).then_some(entry.recommended_vram_gb),
        total_required_bytes: Some(total_required),
        downloaded_file_count: usize_to_u32_saturating(
            model_download_targets(model_dir, entry)
                .iter()
                .filter(|target| file_is_present(&target.target_path))
                .count(),
        ),
        total_file_count,
    }
}

fn model_files(
    model_dir: &Path,
    state: &WorkbenchState,
    entry: &crate::catalog::ModelCatalogEntry,
) -> Vec<ModelDownloadFileRecord> {
    model_download_targets(model_dir, entry)
        .into_iter()
        .map(|target| {
            let download = state
                .downloads
                .values()
                .filter(|item| item.download_key == target.download_key)
                .max_by_key(|item| item.last_event_at.clone());
            let exists = file_is_present(&target.target_path);
            let status = file_status(exists, download);
            let downloaded = if exists {
                target.total_bytes
            } else {
                download.map_or(0, |item| item.downloaded_bytes)
            };
            let bytes_per_second = download.map_or(0.0, download_rate);
            ModelDownloadFileRecord {
                file_id: target.file_id,
                file_name: target.file_name,
                status,
                local_path: exists.then(|| target.target_path.to_string_lossy().to_string()),
                downloaded_bytes: downloaded,
                total_bytes: Some(target.total_bytes),
                percent: download_percent(downloaded, target.total_bytes),
                bytes_per_second,
                eta_seconds: eta_seconds(
                    target.total_bytes.saturating_sub(downloaded),
                    bytes_per_second,
                ),
                error: download.and_then(|item| item.error.clone()),
                error_kind: download.and_then(|item| item.error_kind.clone()),
            }
        })
        .collect()
}

fn model_downloads<'a>(state: &'a WorkbenchState, model_id: &str) -> Vec<&'a DownloadState> {
    state
        .downloads
        .values()
        .filter(|download| download.owner_kind == "model" && download.owner_id == model_id)
        .collect()
}

fn file_status(exists: bool, download: Option<&DownloadState>) -> String {
    if let Some(download) = download.filter(|item| is_active_download_status(&item.status)) {
        return download.status.clone();
    }
    if exists {
        return "downloaded".to_string();
    }
    download
        .filter(|item| matches!(item.status.as_str(), "failed" | "cancelled")).map_or_else(|| "missing".to_string(), |item| item.status.clone())
}

fn model_status_from_files(files: &[ModelDownloadFileRecord]) -> String {
    if files.iter().any(|file| file.status == "downloading" || file.status == "cancelling") {
        return "downloading".to_string();
    }
    if files.iter().any(|file| file.status == "queued") {
        return "queued".to_string();
    }
    if files.iter().any(|file| file.status == "failed") {
        return "failed".to_string();
    }
    if files.iter().any(|file| file.status == "cancelled") {
        return "cancelled".to_string();
    }
    if files.iter().all(|file| file.status == "downloaded") {
        return "downloaded".to_string();
    }
    "missing".to_string()
}

#[allow(
    clippy::cast_precision_loss,
    reason = "download progress percentages are approximate UI telemetry"
)]
fn download_percent(downloaded_bytes: u64, total_bytes: u64) -> f64 {
    if total_bytes == 0 {
        0.0
    } else {
        downloaded_bytes as f64 / total_bytes as f64 * 100.0
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "download throughput is approximate UI telemetry"
)]
fn download_rate(download: &DownloadState) -> f64 {
    download
        .started_at
        .filter(|_| download.status == "downloading")
        .map_or(0.0, |started| {
            download.downloaded_bytes as f64 / started.elapsed().as_secs_f64().max(1.0)
        })
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "download ETA is an approximate positive second count for UI display"
)]
fn eta_seconds(remaining_bytes: u64, bytes_per_second: f64) -> Option<u64> {
    (bytes_per_second.is_finite() && bytes_per_second > 0.0)
        .then(|| (remaining_bytes as f64 / bytes_per_second).ceil() as u64)
}

fn model_status_message(status: &str) -> &'static str {
    match status {
        "downloaded" => "Model files are present. Scans can start.",
        "downloading" => "Downloading a required model file.",
        "queued" => "A required model file is queued.",
        "failed" => "A required file failed to download. Retry from the model library.",
        "cancelled" => "Download was cancelled. Retry from the model library.",
        _ => "One or more required files are missing. Download to restore them.",
    }
}
