fn model_record(
    model_dir: &Path,
    state: &WorkbenchState,
    entry: &crate::catalog::ModelCatalogEntry,
) -> ModelAssetRecord {
    let model_path = model_dir.join(entry.model_file);
    let mmproj_path = model_dir.join(SHARED_MMPROJ_FILE);
    let model_ready = model_path.is_file() && mmproj_path.is_file();
    let download = state.downloads.get(entry.model_id);
    let status = if model_ready {
        "downloaded".to_string()
    } else {
        download
            .map(|item| item.status.clone())
            .unwrap_or_else(|| "missing".to_string())
    };
    let downloaded_bytes = download.map(|item| item.downloaded_bytes).unwrap_or(0);
    let total = download
        .and_then(|item| item.total_bytes)
        .or(Some(entry.model_size_bytes + SHARED_MMPROJ_SIZE_BYTES));
    let percent = total
        .filter(|value| *value > 0)
        .map(|value| downloaded_bytes as f64 / value as f64 * 100.0)
        .unwrap_or(0.0);
    let bytes_per_second = download
        .and_then(|item| item.started_at)
        .map(|started| downloaded_bytes as f64 / started.elapsed().as_secs_f64().max(1.0))
        .unwrap_or(0.0);
    ModelAssetRecord {
        model_id: entry.model_id.to_string(),
        display_name: entry.display_name.to_string(),
        status,
        repo_id: PROVIDER_REPO_ID.to_string(),
        revision: PROVIDER_REVISION.to_string(),
        local_path: model_ready.then(|| model_path.to_string_lossy().to_string()),
        size_bytes: Some(entry.model_size_bytes),
        error: download.and_then(|item| item.error.clone()),
        model_file: entry.model_file.to_string(),
        mmproj_file: SHARED_MMPROJ_FILE.to_string(),
        current_file: download.and_then(|item| item.current_file.clone()),
        status_message: None,
        downloaded_bytes,
        total_bytes: total,
        overall_downloaded_bytes: if model_ready {
            entry.model_size_bytes + SHARED_MMPROJ_SIZE_BYTES
        } else {
            downloaded_bytes
        },
        overall_total_bytes: Some(entry.model_size_bytes + SHARED_MMPROJ_SIZE_BYTES),
        overall_percent: if model_ready { 100.0 } else { percent },
        bytes_per_second,
        eta_seconds: total.and_then(|value| {
            let remaining = value.saturating_sub(downloaded_bytes);
            (bytes_per_second > 0.0).then(|| (remaining as f64 / bytes_per_second) as u64)
        }),
        auth_available: hf_token().is_some(),
        auth_source: hf_token().map(|_| "environment".to_string()),
        last_event_at: download.and_then(|item| item.last_event_at.clone()),
        files: model_files(model_dir, entry, download),
        quantization: entry.quantization.to_string(),
        bits: entry.bits,
        quality: entry.quality.to_string(),
        hardware_tier: entry.hardware_tier.to_string(),
        notes: entry.notes.to_string(),
        recommended: entry.recommended,
        selected: state.selected_model_id == entry.model_id,
        provider_name: PROVIDER_LABEL.to_string(),
        total_required_bytes: Some(entry.model_size_bytes + SHARED_MMPROJ_SIZE_BYTES),
        downloaded_file_count: u32::from(model_path.is_file()) + u32::from(mmproj_path.is_file()),
        total_file_count: 2,
    }
}

fn model_files(
    model_dir: &Path,
    entry: &crate::catalog::ModelCatalogEntry,
    download: Option<&DownloadState>,
) -> Vec<ModelDownloadFileRecord> {
    [
        ("model", entry.model_file, entry.model_size_bytes),
        ("mmproj", SHARED_MMPROJ_FILE, SHARED_MMPROJ_SIZE_BYTES),
    ]
    .into_iter()
    .map(|(file_id, file_name, total)| {
        let local_path = model_dir.join(file_name);
        let exists = local_path.is_file();
        let is_current = download
            .and_then(|item| item.current_file.as_deref())
            .is_some_and(|current| current == file_name);
        let downloaded = if exists {
            total
        } else if is_current {
            download.map(|item| item.downloaded_bytes).unwrap_or(0)
        } else {
            0
        };
        ModelDownloadFileRecord {
            file_id: file_id.to_string(),
            file_name: file_name.to_string(),
            status: if exists {
                "downloaded".to_string()
            } else if is_current {
                "downloading".to_string()
            } else {
                "missing".to_string()
            },
            local_path: exists.then(|| local_path.to_string_lossy().to_string()),
            downloaded_bytes: downloaded,
            total_bytes: Some(total),
            percent: downloaded as f64 / total as f64 * 100.0,
            bytes_per_second: 0.0,
            eta_seconds: None,
            error: None,
        }
    })
    .collect()
}
