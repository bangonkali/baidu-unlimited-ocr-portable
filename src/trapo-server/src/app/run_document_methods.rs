impl AppState {
    pub(crate) async fn list_runs(&self) -> IngestRunsPayload {
        let state = self.inner.state.lock().await;
        IngestRunsPayload {
            runs: state.runs.values().rev().map(run_record).collect(),
        }
    }

    pub(crate) async fn get_run(&self, run_id: &str) -> Result<IngestRunRecord> {
        let state = self.inner.state.lock().await;
        state
            .runs
            .get(run_id)
            .map(run_record)
            .ok_or_else(|| AppError::NotFound("run not found".to_string()))
    }

    pub(crate) async fn stop_run(&self, run_id: &str) -> Result<IngestRunRecord> {
        let (record, document_events, run_to_store, documents_to_store) = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.get_mut(run_id) else {
                return Err(AppError::NotFound("run not found".to_string()));
            };
            if run.completion_manifest.is_some() {
                return Err(AppError::Conflict(
                    "completed runs cannot be stopped; restart from the ingest start page"
                        .to_string(),
                ));
            }
            if !run_is_active(&run.status) {
                return Ok(run_record(run));
            }
            run.cancel_requested = true;
            run.status = "cancelled".to_string();
            let file_hashes = run.file_hashes.clone();
            let run_to_store = stored_run(run);
            let record = run_record(run);
            let mut document_events = Vec::new();
            let mut documents_to_store = Vec::new();
            for file_hash in file_hashes {
                if let Some(document) = state.documents.get_mut(&file_hash) {
                    if matches!(document.status.as_str(), "queued" | "running" | "rendering") {
                        document.status = "cancelled".to_string();
                    }
                    documents_to_store.push(stored_document(document));
                    document_events.push(document_summary(document));
                }
            }
            (record, document_events, run_to_store, documents_to_store)
        };
        self.inner.repository.upsert_run(&run_to_store).await?;
        for document in &documents_to_store {
            self.inner.repository.upsert_document(document).await?;
        }
        self.log_warn("ingest", format!("stop requested for run {run_id}"));
        self.inner
            .hub
            .publish("run.changed", serde_json::to_value(&record)?);
        for event in document_events {
            self.inner
                .hub
                .publish("document.changed", serde_json::to_value(event)?);
        }
        self.publish_status_changed().await;
        Ok(record)
    }

    pub(crate) async fn run_metrics(
        &self,
        run_id: Option<&str>,
        limit: u32,
    ) -> Result<OcrMetricsTreePayload> {
        let rows = self.inner.repository.list_page_metrics(run_id, limit).await?;
        Ok(metrics_tree(rows))
    }

    pub(crate) async fn preview_results(
        &self,
        run_id: &str,
        file_hash: &str,
    ) -> Result<IngestPreviewResultsPayload> {
        let rows = self
            .inner
            .repository
            .preview_results_for_document(run_id, file_hash)
            .await?;
        Ok(IngestPreviewResultsPayload {
            run_id: run_id.to_string(),
            file_hash: file_hash.to_string(),
            results: rows.into_iter().map(preview_result_record).collect(),
        })
    }

    pub(crate) async fn list_documents(&self, query: Option<String>) -> Result<DocumentsPayload> {
        let persisted = if let Some(query) = query.filter(|value| !value.is_empty()) {
            Some(
                self.inner
                    .repository
                    .search_document_hashes(&query, 200)
                    .await?,
            )
        } else {
            None
        };
        let state = self.inner.state.lock().await;
        let documents = persisted.as_ref().map_or_else(
            || state.documents.values().map(document_summary).collect(),
            |persisted| {
                persisted
                    .iter()
                    .filter_map(|hash| state.documents.get(hash).map(document_summary))
                    .collect()
            },
        );
        drop(state);
        Ok(DocumentsPayload { documents })
    }

    pub(crate) async fn get_document(&self, file_hash: &str) -> Result<DocumentDetail> {
        let detail = {
            let state = self.inner.state.lock().await;
            let document = state
                .documents
                .get(file_hash)
                .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
            let detail = document_detail(document);
            drop(state);
            detail
        };
        Ok(detail)
    }

    pub(crate) async fn document_regions(
        &self,
        file_hash: &str,
        run_id: Option<&str>,
        run_engine_id: Option<&str>,
    ) -> Result<DocumentRegionsPayload> {
        if let Some(run_engine_id) = run_engine_id.filter(|value| !value.is_empty()) {
            return Ok(DocumentRegionsPayload {
                file_hash: file_hash.to_string(),
                run_id: run_id.map(ToString::to_string),
                run_engine_id: Some(run_engine_id.to_string()),
                boxes: self
                    .inner
                    .repository
                    .load_document_regions_for_run_engine(file_hash, run_engine_id)
                    .await?,
            });
        }
        if let Some(run_id) = run_id.filter(|value| !value.is_empty()) {
            return Ok(DocumentRegionsPayload {
                file_hash: file_hash.to_string(),
                run_id: Some(run_id.to_string()),
                run_engine_id: None,
                boxes: self
                    .inner
                    .repository
                    .load_document_regions_for_run(file_hash, run_id)
                    .await?,
            });
        }
        let state = self.inner.state.lock().await;
        let boxes = state
            .documents
            .get(file_hash)
            .map(|document| {
                document
                    .pages
                    .iter()
                    .flat_map(|page| page.boxes.clone())
                    .collect()
            })
            .unwrap_or_default();
        drop(state);
        Ok(DocumentRegionsPayload {
            file_hash: file_hash.to_string(),
            run_id: None,
            run_engine_id: None,
            boxes,
        })
    }

    pub(crate) async fn document_text(
        &self,
        file_hash: &str,
        run_id: Option<&str>,
        run_engine_id: Option<&str>,
    ) -> Result<DocumentTextPayload> {
        if let Some(run_engine_id) = run_engine_id.filter(|value| !value.is_empty()) {
            return Ok(DocumentTextPayload {
                file_hash: file_hash.to_string(),
                run_id: run_id.map(ToString::to_string),
                run_engine_id: Some(run_engine_id.to_string()),
                pages: self
                    .inner
                    .repository
                    .load_document_text_for_run_engine(file_hash, run_engine_id)
                    .await?,
            });
        }
        if let Some(run_id) = run_id.filter(|value| !value.is_empty()) {
            return Ok(DocumentTextPayload {
                file_hash: file_hash.to_string(),
                run_id: Some(run_id.to_string()),
                run_engine_id: None,
                pages: self
                    .inner
                    .repository
                    .load_document_text_for_run(file_hash, run_id)
                    .await?,
            });
        }
        let state = self.inner.state.lock().await;
        let pages = state
            .documents
            .get(file_hash)
            .map(started_page_text_records)
            .unwrap_or_default();
        drop(state);
        Ok(DocumentTextPayload {
            file_hash: file_hash.to_string(),
            run_id: None,
            run_engine_id: None,
            pages,
        })
    }

    pub(crate) async fn preview_images(&self, file_hash: &str) -> PreviewImagesPayload {
        let state = self.inner.state.lock().await;
        let pages = state
            .documents
            .get(file_hash)
            .map(|document| {
                if document.pages.is_empty() {
                    Vec::new()
                } else {
                    document.pages.iter().map(|page| page.page_no).collect()
                }
            })
            .unwrap_or_default();
        drop(state);
        PreviewImagesPayload {
            file_hash: file_hash.to_string(),
            variants: if pages.is_empty() {
                Vec::new()
            } else {
                vec!["source".to_string()]
            },
            pages,
        }
    }

    pub(crate) async fn preview_image_path(
        &self,
        file_hash: &str,
        variant: &str,
        page_no: u32,
    ) -> Option<PathBuf> {
        if variant != "source" {
            return None;
        }
        let state = self.inner.state.lock().await;
        let path = state.documents.get(file_hash).and_then(|document| {
            document
                .pages
                .iter()
                .find(|page| page.page_no == page_no)
                .map(|page| page.image_path.clone())
        });
        drop(state);
        path
    }
}
