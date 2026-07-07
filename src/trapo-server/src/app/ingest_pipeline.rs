impl AppState {
    fn spawn_ingest(&self, execution: IngestExecution) {
        let state = self.clone();
        self.spawn_background(async move {
            state.run_ingest(execution).await;
        });
    }

    async fn run_ingest(&self, execution: IngestExecution) {
        let IngestExecution {
            completed_pages,
            embedding_after_ingest,
            embedding_dimension,
            embedding_model_id,
            engine_configs,
            files,
            run_id,
            text_index_after_ingest,
        } = execution;
        self.mark_run_status(&run_id, "running", None).await;
        for engine_config in engine_configs {
            if self.run_cancelled(&run_id).await {
                break;
            }
            self.run_ingest_engine(&run_id, &files, &completed_pages, engine_config)
                .await;
        }
        let final_status = self.finish_ingest_run(&run_id).await;
        if final_status == "completed" {
            self.run_post_ingest_pipeline(
                &run_id,
                text_index_after_ingest,
                embedding_after_ingest,
                embedding_model_id,
                embedding_dimension,
            )
            .await;
        }
    }

    async fn run_ingest_engine(
        &self,
        run_id: &str,
        files: &[DiscoveredFile],
        completed_pages: &BTreeSet<(String, u32)>,
        engine_config: RunEngineConfigState,
    ) {
        let profile_id = engine_config.profile_id.clone().unwrap_or_default();
        let model_id = engine_config.model_id.clone().unwrap_or_default();
        let runtime_id = engine_config.runtime_id.clone().unwrap_or_default();
        self.mark_engine_status(&engine_config.run_engine_id, "running", None, 0)
            .await;
        let lease_scope = DiagnosticSpanScope::start();
        let ocr_worker = self
            .create_ocr_worker(&engine_config.engine_id, &runtime_id, &profile_id, &model_id)
            .await;
        let fallback_reason = ocr_worker.fallback_error().map(ToString::to_string);
        self.record_model_lease(
            ModelLeaseDiagnostic {
                run_id,
                model_id: &model_id,
                runtime_id: &runtime_id,
                profile_id: &profile_id,
                error: fallback_reason.as_deref(),
            },
            &lease_scope,
        );
        if let Some(reason) = fallback_reason.as_deref() {
            self.log_warn(
                "ocr",
                format!("using fallback output for {}: {reason}", engine_config.engine_id),
            );
        }
        let (runtime_platform, accelerator) = self.runtime_stream_metadata(&runtime_id).await;
        let ocr_context = OcrRunContext {
            run_engine_id: &engine_config.run_engine_id,
            engine_kind: &engine_config.engine_kind,
            engine_id: &engine_config.engine_id,
            profile_id: &profile_id,
            model_id: &model_id,
            runtime_id: &runtime_id,
            runtime_platform: &runtime_platform,
            accelerator: &accelerator,
            worker: &ocr_worker,
        };
        let mut output_count = 0_u32;
        let mut failed = false;
        for file in files {
            if self.run_cancelled(run_id).await {
                break;
            }
            let file_hash = stable_hash(file);
            match self
                .process_document(run_id, &file_hash, &ocr_context, completed_pages)
                .await
            {
                Ok(()) => {
                    output_count = output_count.saturating_add(1);
                }
                Err(error) => {
                    failed = true;
                    self.record_diagnostic_event(
                        run_id,
                        Some(&file_hash),
                        None,
                        "error",
                        &error.to_string(),
                    );
                    self.mark_document_error(run_id, &file_hash, error.to_string())
                        .await;
                }
            }
        }
        let status = engine_completion_status(output_count, failed, fallback_reason.is_some());
        self.mark_engine_status(
            &engine_config.run_engine_id,
            status,
            fallback_reason.as_deref(),
            output_count,
        )
        .await;
    }

    async fn finish_ingest_run(&self, run_id: &str) -> String {
        let final_status = if self.run_cancelled(run_id).await {
            "cancelled"
        } else if self.run_has_errors(run_id).await {
            "completed_with_errors"
        } else {
            "completed"
        };
        self.mark_run_status(run_id, final_status, None).await;
        if final_status == "completed"
            && let Err(error) = self.persist_run_completion_manifest(run_id).await
        {
            self.log_warn(
                "ingest",
                format!("failed to write completion manifest for run {run_id}: {error}"),
            );
        }
        {
            let mut state = self.inner.state.lock().await;
            if state.active_run_id.as_deref() == Some(run_id) {
                state.active_run_id = None;
            }
        }
        self.publish_status_changed().await;
        final_status.to_string()
    }

    async fn run_post_ingest_pipeline(
        &self,
        run_id: &str,
        text_index_after_ingest: bool,
        embedding_after_ingest: bool,
        embedding_model_id: Option<String>,
        embedding_dimension: Option<u32>,
    ) {
        if text_index_after_ingest
            && let Err(error) = self
                .start_text_index(TextIndexRequest {
                    source_run_id: run_id.to_string(),
                })
                .await
        {
            self.log_warn("rag", format!("post-ingest text index failed: {error}"));
            return;
        }
        if embedding_after_ingest
            && let Some(model_id) = embedding_model_id
            && let Err(error) = self
                .start_generate_embedding(GenerateEmbeddingRequest {
                    source_run_id: run_id.to_string(),
                    model_id,
                    dimension: embedding_dimension,
                })
                .await
        {
            self.log_warn("rag", format!("post-ingest embedding failed: {error}"));
        }
    }
}

const fn engine_completion_status(output_count: u32, failed: bool, fallback: bool) -> &'static str {
    if output_count == 0 && failed {
        "failed"
    } else if failed || fallback {
        "completed_with_errors"
    } else {
        "completed"
    }
}

struct IngestExecution {
    completed_pages: BTreeSet<(String, u32)>,
    embedding_after_ingest: bool,
    embedding_dimension: Option<u32>,
    embedding_model_id: Option<String>,
    engine_configs: Vec<RunEngineConfigState>,
    files: Vec<DiscoveredFile>,
    run_id: String,
    text_index_after_ingest: bool,
}
