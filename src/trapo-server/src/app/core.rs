impl AppState {
    /// Creates server state and opens required local resources.
    ///
    /// # Errors
    ///
    /// Returns an error when directories, the logger, or the `DuckDB` repository
    /// cannot be initialized.
    pub async fn new(config: ServerConfig) -> Result<Self> {
        config.ensure_directories()?;
        let repository = Repository::open(config.database_path.clone()).await?;
        let logger = AppLogger::open(&config.log_dir)?;
        let hub = RealtimeHub::new();
        hub.attach_repository(repository.clone());
        let annotation_identities = AnnotationIdentityRuntime::new(repository.clone());
        let variants = runtime_variants(&config.app_root);
        let selected_model_id =
            read_string_setting(&repository, "selected_model_id", DEFAULT_MODEL_ID).await;
        let selected_profile_id =
            read_string_setting(&repository, "selected_profile_id", DEFAULT_PROFILE_ID).await;
        let preferred_runtime = read_string_setting(&repository, "selected_runtime_id", "").await;
        let selected_runtime_id = choose_runtime_id(&variants, &preferred_runtime);
        let workbench_ui = repository
            .setting_value("workbench_ui")
            .await?
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_default();
        let renderer = PdfRenderer::new(
            config.cache_dir.join("pdfium"),
            config.pdfium_library_dir.clone(),
            PDF_DPI,
        );
        let mut state = WorkbenchState {
            selected_model_id: selected_model_id.clone(),
            selected_profile_id: selected_profile_id.clone(),
            selected_runtime_id,
            runtime_variants: variants,
            workbench_ui,
            active_run_id: None,
            runs: BTreeMap::new(),
            documents: BTreeMap::new(),
            downloads: HashMap::new(),
            download_queue: VecDeque::new(),
        };
        hydrate_snapshot(&repository, &mut state).await?;
        let app = Self {
            inner: Arc::new(AppInner {
                config,
                repository,
                logger,
                hub,
                renderer,
                annotation_identities,
                background_tasks: BackgroundTasks::default(),
                shutdown: ShutdownCoordinator::new(),
                state: Mutex::new(state),
            }),
        };
        app.log_info("server", "trapo-server initialized");
        Ok(app)
    }

    /// Returns the effective server configuration.
    #[must_use]
    pub fn config(&self) -> &ServerConfig {
        &self.inner.config
    }

    #[must_use]
    pub(crate) fn hub(&self) -> Arc<RealtimeHub> {
        self.inner.hub.clone()
    }

    pub(crate) fn health() -> HealthPayload {
        HealthPayload {
            ok: true,
            service: "trapo-server".to_string(),
        }
    }

    pub(crate) async fn status(&self) -> StatusPayload {
        let state = self.inner.state.lock().await;
        let runtime = selected_runtime(&state);
        StatusPayload {
            state: if self.inner.shutdown.is_requested() {
                "shutting_down".to_string()
            } else if state.active_run_id.is_some() {
                "running".to_string()
            } else {
                "idle".to_string()
            },
            host: self.inner.config.host.clone(),
            active_run_id: state.active_run_id.clone(),
            default_profile: state.selected_profile_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_tag: option_env!("TRAPO_GIT_TAG").unwrap_or("dev").to_string(),
            git_sha: option_env!("TRAPO_GIT_SHA")
                .unwrap_or("unknown")
                .to_string(),
            supported_inputs: SUPPORTED_INPUTS
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            runtime_platform: runtime.map(|item| item.platform.clone()),
            accelerator: runtime.map(|item| item.accelerator.clone()),
            runtime_selectable: runtime.is_some_and(|item| item.selectable),
            runtime_variants: state
                .runtime_variants
                .iter()
                .map(|item| runtime_record(item, &state.selected_runtime_id))
                .collect(),
            inference_engine: "Unlimited-OCR FFI".to_string(),
            log_path: self.inner.logger.path().to_string_lossy().to_string(),
            database_path: self.inner.repository.path().to_string_lossy().to_string(),
            realtime_path: "/api/events".to_string(),
            selected_model_id: state.selected_model_id.clone(),
        }
    }

    pub(crate) fn logs(&self, limit: usize) -> LogsPayload {
        self.inner.logger.recent(limit)
    }

    pub(crate) async fn folder_dialog(&self) -> FolderDialogResponse {
        crate::folder_dialog::open_folder_dialog().await
    }

    pub(crate) async fn settings(&self) -> SettingsPayload {
        let state = self.inner.state.lock().await;
        settings_payload(&self.inner, &state)
    }

    pub(crate) async fn update_settings(
        &self,
        request: SettingsUpdateRequest,
    ) -> Result<SettingsPayload> {
        let mut settings_to_persist = Vec::new();
        let payload = {
            let mut state = self.inner.state.lock().await;
            if let Some(runtime_id) = request
                .selected_runtime_id
                .filter(|value| !value.is_empty())
            {
                let runtime = state
                    .runtime_variants
                    .iter()
                    .find(|item| item.runtime_id == runtime_id);
                if !runtime.is_some_and(|item| item.selectable) {
                    return Err(AppError::BadRequest(format!(
                        "runtime is not supported on this device or is not installed: {runtime_id}"
                    )));
                }
                state.selected_runtime_id.clone_from(&runtime_id);
                settings_to_persist.push(("selected_runtime_id", Value::String(runtime_id)));
            }
            if let Some(profile_id) = request.default_profile.filter(|value| !value.is_empty()) {
                if find_profile(&profile_id).is_none() {
                    return Err(AppError::BadRequest(format!(
                        "unknown OCR profile: {profile_id}"
                    )));
                }
                state.selected_profile_id.clone_from(&profile_id);
                settings_to_persist.push(("selected_profile_id", Value::String(profile_id)));
            }
            if let Some(patch) = request.workbench_ui {
                apply_workbench_patch(&mut state.workbench_ui, patch)?;
                settings_to_persist.push((
                    "workbench_ui",
                    serde_json::to_value(&state.workbench_ui)?,
                ));
            }
            let payload = settings_payload(&self.inner, &state);
            drop(state);
            payload
        };
        for (key, value) in settings_to_persist {
            self.inner.repository.put_setting(key, &value).await?;
        }
        self.publish_status_changed().await;
        Ok(payload)
    }
}
