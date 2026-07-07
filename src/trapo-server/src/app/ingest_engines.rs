impl AppState {
    pub(crate) async fn ingest_engines(&self) -> IngestEnginesPayload {
        let state = self.inner.state.lock().await;
        let selected_runtime_id = state.selected_runtime_id.clone();
        drop(state);
        IngestEnginesPayload {
            engines: engine_preset_definitions()
                .into_iter()
                .map(|preset| self.engine_preset_record(preset, &selected_runtime_id))
                .collect(),
        }
    }

    async fn resolve_ingest_engine_configs(
        &self,
        request: &IngestStartRequest,
        run_id: &str,
    ) -> Result<Vec<RunEngineConfigState>> {
        let selections = self.ingest_engine_selections(request).await?;
        let mut configs = Vec::with_capacity(selections.len());
        for (index, selection) in selections.into_iter().enumerate() {
            let preset = find_engine_preset(&selection)?;
            let config = self
                .resolve_engine_selection(run_id, index, selection, preset)
                .await?;
            configs.push(config);
        }
        Ok(configs)
    }

    async fn ingest_engine_selections(
        &self,
        request: &IngestStartRequest,
    ) -> Result<Vec<crate::workbench_types::IngestEngineSelection>> {
        if let Some(engines) = &request.engines {
            if engines.is_empty() {
                return Err(AppError::BadRequest(
                    "at least one ingest engine is required".to_string(),
                ));
            }
            return Ok(engines.clone());
        }
        let (profile_id, model_id, runtime_id) = self.resolve_ingest_selection(request).await?;
        Ok(vec![crate::workbench_types::IngestEngineSelection {
            preset_id: Some("ocr-unlimited-ocr-ffi".to_string()),
            engine_id: request
                .engine_id
                .clone()
                .unwrap_or_else(|| ENGINE_ID.to_string()),
            engine_kind: "ocr".to_string(),
            model_id: Some(model_id),
            profile_id: Some(profile_id),
            runtime_id: Some(runtime_id),
            parameters: Some(json!({})),
            ordinal: Some(0),
        }])
    }

    async fn resolve_engine_selection(
        &self,
        run_id: &str,
        index: usize,
        selection: crate::workbench_types::IngestEngineSelection,
        preset: EnginePresetDefinition,
    ) -> Result<RunEngineConfigState> {
        let state = self.inner.state.lock().await;
        let selected_profile_id = state.selected_profile_id.clone();
        let selected_runtime_id = state.selected_runtime_id.clone();
        let runtime_variants = state.runtime_variants.clone();
        drop(state);

        let engine_kind = if selection.engine_kind.trim().is_empty() {
            preset.engine_kind.to_string()
        } else {
            selection.engine_kind
        };
        if engine_kind != preset.engine_kind {
            return Err(AppError::BadRequest(format!(
                "engine kind mismatch for {}: expected {}",
                preset.engine_id, preset.engine_kind
            )));
        }
        if engine_kind != "ocr" && engine_kind != "document_understanding" {
            return Err(AppError::BadRequest(format!(
                "unsupported engine kind: {engine_kind}"
            )));
        }

        let model_id = selection
            .model_id
            .filter(|value| !value.is_empty())
            .or_else(|| preset.model_id.map(ToString::to_string));
        let profile_id = selection
            .profile_id
            .filter(|value| !value.is_empty())
            .or_else(|| preset.profile_id.map(ToString::to_string))
            .or_else(|| (engine_kind == "ocr").then_some(selected_profile_id));
        let runtime_id = selection
            .runtime_id
            .filter(|value| !value.is_empty())
            .or_else(|| model_id.as_ref().map(|_| selected_runtime_id.clone()));

        validate_engine_model(model_id.as_deref(), &engine_kind)?;
        validate_engine_profile(profile_id.as_deref())?;
        validate_engine_runtime(runtime_id.as_deref(), &runtime_variants)?;
        validate_engine_runner(&preset)?;
        self.validate_engine_downloads(preset.engine_id, model_id.as_deref())?;
        self.validate_engine_runner_binary(
            preset.engine_id,
            runtime_id.as_deref(),
            &selected_runtime_id,
        )?;

        Ok(RunEngineConfigState {
            run_engine_id: new_persistence_id(),
            run_id: run_id.to_string(),
            ordinal: selection
                .ordinal
                .unwrap_or_else(|| u32::try_from(index).unwrap_or(u32::MAX)),
            engine_kind,
            engine_id: preset.engine_id.to_string(),
            model_id,
            profile_id,
            runtime_id,
            parameters: selection
                .parameters
                .unwrap_or_else(|| preset.default_parameters.clone()),
            status: "queued".to_string(),
            error: None,
            usable_output_count: 0,
        })
    }

    fn engine_preset_record(
        &self,
        preset: EnginePresetDefinition,
        selected_runtime_id: &str,
    ) -> IngestEnginePresetRecord {
        let runner = runner_capability(preset.engine_id);
        let (available, availability, availability_detail) =
            self.engine_availability(&preset, selected_runtime_id);
        IngestEnginePresetRecord {
            preset_id: preset.preset_id.to_string(),
            engine_id: preset.engine_id.to_string(),
            engine_kind: preset.engine_kind.to_string(),
            label: preset.label.to_string(),
            description: preset.description.to_string(),
            model_id: preset.model_id.map(ToString::to_string),
            profile_id: preset.profile_id.map(ToString::to_string),
            runtime_id: preset.model_id.map(|_| selected_runtime_id.to_string()),
            previewer: preset.previewer.to_string(),
            default_enabled: available && availability == "ready",
            requires_model: preset.model_id.is_some(),
            download_model_ids: preset.model_id.into_iter().map(ToString::to_string).collect(),
            available,
            availability,
            availability_detail,
            runner_kind: runner.kind.to_string(),
            runner_status: runner.status.to_string(),
            runner_detail: runner.detail.map(ToString::to_string),
            parameter_schema: json!({ "type": "object", "additionalProperties": true }),
            default_parameters: preset.default_parameters,
        }
    }

    fn engine_availability(
        &self,
        preset: &EnginePresetDefinition,
        selected_runtime_id: &str,
    ) -> (bool, String, Option<String>) {
        if let Some(model_id) = preset.model_id
            && !self.model_files_ready(model_id)
        {
            return (
                false,
                "missing_model".to_string(),
                Some(format!("download model files for {model_id}")),
            );
        }
        if preset.model_id.is_some() && selected_runtime_id.is_empty() {
            return (
                false,
                "runtime_unavailable".to_string(),
                Some("select an installed runtime".to_string()),
            );
        }
        if let Some(expected) = expected_runner_binary(preset.engine_id)
            && !runner_binary_is_installed(
                &self.inner.config.app_root,
                selected_runtime_id,
                preset.engine_id,
            )
        {
            return (
                false,
                "native_runner_missing".to_string(),
                Some(format!(
                    "build or install the native runner binary: {expected}"
                )),
            );
        }
        (true, "ready".to_string(), None)
    }

    fn validate_engine_downloads(&self, engine_id: &str, model_id: Option<&str>) -> Result<()> {
        if let Some(model_id) = model_id
            && !self.model_files_ready(model_id)
        {
            return Err(AppError::BadRequest(format!(
                "model files are missing for engine {engine_id}: {model_id}"
            )));
        }
        Ok(())
    }

    fn validate_engine_runner_binary(
        &self,
        engine_id: &str,
        runtime_id: Option<&str>,
        selected_runtime_id: &str,
    ) -> Result<()> {
        let Some(expected) = expected_runner_binary(engine_id) else {
            return Ok(());
        };
        let runtime_id = runtime_id
            .filter(|value| !value.is_empty())
            .unwrap_or(selected_runtime_id);
        if runner_binary_is_installed(&self.inner.config.app_root, runtime_id, engine_id) {
            return Ok(());
        }
        Err(AppError::BadRequest(format!(
            "native runner is missing for engine {engine_id}: expected {expected}"
        )))
    }

    fn model_files_ready(&self, model_id: &str) -> bool {
        let Some(model) = find_model(model_id) else {
            return false;
        };
        model_download_targets(&self.inner.config.model_dir, model)
            .iter()
            .all(|target| file_is_present(&target.target_path))
    }
}
