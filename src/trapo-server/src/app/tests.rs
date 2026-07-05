#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn applies_workbench_patch() -> Result<()> {
        let mut settings = WorkbenchUiSettings::default();
        apply_workbench_patch(
            &mut settings,
            crate::types::WorkbenchUiSettingsPatch {
                theme: Some("light".to_string()),
                overlay_visible: Some(false),
                ..Default::default()
            },
        )?;
        assert_eq!(settings.theme, "light");
        assert!(!settings.overlay_visible);
        Ok(())
    }

    #[test]
    fn model_record_uses_file_download_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let Some(entry) = find_model(DEFAULT_MODEL_ID) else {
            return Err(AppError::Internal("default model is missing".to_string()));
        };
        let targets = model_download_targets(temp.path(), entry);
        let Some(target) = targets.into_iter().find(|target| target.file_id == "model") else {
            return Err(AppError::Internal("model target is missing".to_string()));
        };
        let mut state = test_workbench_state(entry.model_id);
        let download_id = new_persistence_id();
        state.downloads.insert(
            download_id.clone(),
            DownloadState {
                download_id,
                download_key: target.download_key,
                owner_kind: target.owner_kind,
                owner_id: target.owner_id,
                file_id: target.file_id,
                file_name: target.file_name,
                source_url: target.source_url,
                target_path: target.target_path,
                force: false,
                status: "downloading".to_string(),
                downloaded_bytes: 128,
                total_bytes: Some(256),
                error: None,
                started_at: Some(Instant::now()),
                last_progress_publish_at: None,
                last_progress_publish_bytes: 0,
                cancel_requested: false,
                last_event_at: Some("2026-07-03T00:00:00Z".to_string()),
            },
        );

        let record = model_record(temp.path(), &state, entry);

        assert_eq!(record.status, "downloading");
        assert_eq!(record.current_file.as_deref(), Some(entry.model_file));
        assert!(record.files.iter().any(|file| file.status == "downloading"));
        assert!(record.files.iter().any(|file| file.status == "missing"));
        Ok(())
    }

    #[test]
    fn completed_download_state_does_not_hide_missing_file() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let Some(entry) = find_model(DEFAULT_MODEL_ID) else {
            return Err(AppError::Internal("default model is missing".to_string()));
        };
        let targets = model_download_targets(temp.path(), entry);
        let Some(target) = targets.into_iter().find(|target| target.file_id == "model") else {
            return Err(AppError::Internal("model target is missing".to_string()));
        };
        let mut state = test_workbench_state(entry.model_id);
        let download_id = new_persistence_id();
        state.downloads.insert(
            download_id.clone(),
            DownloadState {
                download_id,
                download_key: target.download_key,
                owner_kind: target.owner_kind,
                owner_id: target.owner_id,
                file_id: target.file_id,
                file_name: target.file_name,
                source_url: target.source_url,
                target_path: target.target_path,
                force: false,
                status: "downloaded".to_string(),
                downloaded_bytes: target.total_bytes,
                total_bytes: Some(target.total_bytes),
                error: None,
                started_at: Some(Instant::now()),
                last_progress_publish_at: None,
                last_progress_publish_bytes: 0,
                cancel_requested: false,
                last_event_at: Some("2026-07-03T00:00:00Z".to_string()),
            },
        );

        let record = model_record(temp.path(), &state, entry);

        assert_eq!(record.status, "missing");
        assert!(record.files.iter().any(|file| file.status == "missing"));
        Ok(())
    }

    #[test]
    fn download_progress_publication_is_coalesced_by_time_and_size() {
        let mut download = DownloadState {
            download_id: "download-a".to_string(),
            download_key: "model:model-a:model".to_string(),
            owner_kind: "model".to_string(),
            owner_id: DEFAULT_MODEL_ID.to_string(),
            file_id: "model".to_string(),
            file_name: "model.gguf".to_string(),
            source_url: "https://example.test/model.gguf".to_string(),
            target_path: PathBuf::from("model.gguf"),
            force: false,
            status: "downloading".to_string(),
            downloaded_bytes: 0,
            total_bytes: Some(10 * 1024 * 1024),
            error: None,
            started_at: Some(Instant::now()),
            last_progress_publish_at: None,
            last_progress_publish_bytes: 0,
            cancel_requested: false,
            last_event_at: None,
        };
        let now = Instant::now();

        assert!(should_publish_download_progress(&download, 1024, now));
        mark_download_progress_published(&mut download, 1024, now, "now".to_string());
        assert!(!should_publish_download_progress(
            &download,
            512 * 1024,
            now + Duration::from_millis(100)
        ));
        assert!(should_publish_download_progress(
            &download,
            MODEL_DOWNLOAD_PROGRESS_FLUSH_BYTES + 1024,
            now + Duration::from_millis(100)
        ));
        mark_download_progress_published(
            &mut download,
            MODEL_DOWNLOAD_PROGRESS_FLUSH_BYTES + 1024,
            now + Duration::from_millis(100),
            "later".to_string(),
        );
        assert!(should_publish_download_progress(
            &download,
            MODEL_DOWNLOAD_PROGRESS_FLUSH_BYTES + 2048,
            now + Duration::from_millis(MODEL_DOWNLOAD_PROGRESS_FLUSH_MS + 100)
        ));
    }

    fn test_workbench_state(model_id: &str) -> WorkbenchState {
        WorkbenchState {
            selected_model_id: model_id.to_string(),
            selected_profile_id: DEFAULT_PROFILE_ID.to_string(),
            selected_runtime_id: String::new(),
            runtime_variants: Vec::new(),
            workbench_ui: WorkbenchUiSettings::default(),
            active_run_id: None,
            runs: BTreeMap::new(),
            documents: BTreeMap::new(),
            downloads: HashMap::new(),
            download_queue: VecDeque::new(),
        }
    }
}
