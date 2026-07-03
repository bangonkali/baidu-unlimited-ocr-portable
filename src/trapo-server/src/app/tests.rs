#[cfg(test)]
mod tests {
    use super::*;

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
        state.downloads.insert(
            target.download_id.clone(),
            DownloadState {
                download_id: target.download_id,
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
        state.downloads.insert(
            target.download_id.clone(),
            DownloadState {
                download_id: target.download_id,
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
                cancel_requested: false,
                last_event_at: Some("2026-07-03T00:00:00Z".to_string()),
            },
        );

        let record = model_record(temp.path(), &state, entry);

        assert_eq!(record.status, "missing");
        assert!(record.files.iter().any(|file| file.status == "missing"));
        Ok(())
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
