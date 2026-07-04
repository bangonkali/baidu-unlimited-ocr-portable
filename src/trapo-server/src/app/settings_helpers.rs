async fn read_string_setting(repository: &Repository, key: &str, fallback: &str) -> String {
    repository
        .setting_value(key)
        .await
        .ok()
        .flatten()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

async fn hydrate_snapshot(repository: &Repository, state: &mut WorkbenchState) -> Result<()> {
    let snapshot = repository.load_snapshot().await?;
    for run in snapshot.runs {
        state.runs.insert(run.run_id.clone(), run_from_stored(run));
    }
    for run_document in snapshot.run_documents {
        if let Some(run) = state.runs.get_mut(&run_document.run_id) {
            let ordinal = usize::try_from(run_document.ordinal).unwrap_or(usize::MAX);
            let insert_at = ordinal.min(run.file_hashes.len());
            run.file_hashes.insert(insert_at, run_document.file_hash);
        }
    }
    for document in snapshot.documents {
        state
            .documents
            .insert(document.file_hash.clone(), document_from_stored(document));
    }
    for page in snapshot.pages {
        if let Some(document) = state.documents.get_mut(&page.file_hash) {
            document.pages.push(page_from_stored(page));
        }
    }
    Ok(())
}

fn settings_payload(inner: &AppInner, state: &WorkbenchState) -> SettingsPayload {
    let runtime = selected_runtime(state);
    SettingsPayload {
        pdf_dpi: PDF_DPI,
        ocr_concurrency: 1,
        default_profile: state.selected_profile_id.clone(),
        retry_profile: RETRY_PROFILE_ID.to_string(),
        cache_path: inner.config.cache_dir.to_string_lossy().to_string(),
        database_path: inner.repository.path().to_string_lossy().to_string(),
        selected_runtime_id: state.selected_runtime_id.clone(),
        selected_accelerator: runtime
            .map(|item| item.accelerator.clone())
            .unwrap_or_else(|| "cpu".to_string()),
        selected_model_id: state.selected_model_id.clone(),
        runtime_variants: state
            .runtime_variants
            .iter()
            .map(|item| runtime_record(item, &state.selected_runtime_id))
            .collect(),
        workbench_ui: state.workbench_ui.clone(),
    }
}

fn apply_workbench_patch(
    settings: &mut WorkbenchUiSettings,
    patch: crate::types::WorkbenchUiSettingsPatch,
) -> Result<()> {
    if let Some(theme) = patch.theme {
        if theme != "dark" && theme != "light" {
            return Err(AppError::BadRequest(
                "theme must be dark or light".to_string(),
            ));
        }
        settings.theme = theme;
    }
    if let Some(value) = patch.auto_follow_regions {
        settings.auto_follow_regions = value;
    }
    if let Some(value) = patch.labels_visible {
        settings.labels_visible = value;
    }
    if let Some(value) = patch.overlay_visible {
        settings.overlay_visible = value;
    }
    if let Some(panes) = patch.panes_collapsed {
        if let Some(value) = panes.details {
            settings.panes_collapsed.details = value;
        }
        if let Some(value) = panes.diagnostics {
            settings.panes_collapsed.diagnostics = value;
        }
        if let Some(value) = panes.explorer {
            settings.panes_collapsed.explorer = value;
        }
    }
    Ok(())
}

fn selected_runtime(state: &WorkbenchState) -> Option<&crate::catalog::RuntimeVariant> {
    state
        .runtime_variants
        .iter()
        .find(|item| item.runtime_id == state.selected_runtime_id)
}
