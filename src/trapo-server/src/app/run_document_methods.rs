impl AppState {
    pub async fn list_runs(&self) -> IngestRunsPayload {
        let state = self.inner.state.lock().await;
        IngestRunsPayload {
            runs: state.runs.values().rev().map(run_record).collect(),
        }
    }

    pub async fn get_run(&self, run_id: &str) -> Result<IngestRunRecord> {
        let state = self.inner.state.lock().await;
        state
            .runs
            .get(run_id)
            .map(run_record)
            .ok_or_else(|| AppError::NotFound("run not found".to_string()))
    }

    pub async fn stop_run(&self, run_id: &str) -> Result<IngestRunRecord> {
        let (record, document_events) = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.get_mut(run_id) else {
                return Err(AppError::NotFound("run not found".to_string()));
            };
            run.cancel_requested = true;
            run.status = "cancelled".to_string();
            let file_hashes = run.file_hashes.clone();
            self.inner.repository.upsert_run(&stored_run(run))?;
            let record = run_record(run);
            let mut document_events = Vec::new();
            for file_hash in file_hashes {
                if let Some(document) = state.documents.get_mut(&file_hash) {
                    if matches!(document.status.as_str(), "queued" | "running" | "rendering") {
                        document.status = "cancelled".to_string();
                    }
                    self.inner
                        .repository
                        .upsert_document(&stored_document(document))?;
                    document_events.push(document_summary(document));
                }
            }
            (record, document_events)
        };
        self.log_warn("ingest", format!("stop requested for run {run_id}"))
            .await;
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

    pub async fn run_metrics(
        &self,
        run_id: Option<&str>,
        limit: u32,
    ) -> Result<OcrMetricsTreePayload> {
        let rows = self.inner.repository.list_page_metrics(run_id, limit)?;
        Ok(metrics_tree(rows))
    }

    pub async fn list_documents(&self, query: Option<String>) -> Result<DocumentsPayload> {
        let state = self.inner.state.lock().await;
        let documents = if let Some(query) = query.filter(|value| !value.is_empty()) {
            let persisted = self.inner.repository.search_document_hashes(&query, 200)?;
            persisted
                .iter()
                .filter_map(|hash| state.documents.get(hash).map(document_summary))
                .collect()
        } else {
            state.documents.values().map(document_summary).collect()
        };
        Ok(DocumentsPayload { documents })
    }

    pub async fn get_document(&self, file_hash: &str) -> Result<DocumentDetail> {
        let state = self.inner.state.lock().await;
        let document = state
            .documents
            .get(file_hash)
            .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
        Ok(document_detail(document))
    }

    pub async fn document_regions(&self, file_hash: &str) -> DocumentRegionsPayload {
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
        DocumentRegionsPayload {
            file_hash: file_hash.to_string(),
            boxes,
        }
    }

    pub async fn document_text(&self, file_hash: &str) -> DocumentTextPayload {
        let state = self.inner.state.lock().await;
        let pages = state
            .documents
            .get(file_hash)
            .map(|document| {
                document
                    .pages
                    .iter()
                    .map(|page| PageTextRecord {
                        page_no: page.page_no,
                        text: page.cleaned_text.clone(),
                        spans: page.spans.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        DocumentTextPayload {
            file_hash: file_hash.to_string(),
            pages,
        }
    }

    pub async fn preview_images(&self, file_hash: &str) -> PreviewImagesPayload {
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

    pub async fn preview_image_path(
        &self,
        file_hash: &str,
        variant: &str,
        page_no: u32,
    ) -> Option<PathBuf> {
        if variant != "source" {
            return None;
        }
        let state = self.inner.state.lock().await;
        state.documents.get(file_hash).and_then(|document| {
            document
                .pages
                .iter()
                .find(|page| page.page_no == page_no)
                .map(|page| page.image_path.clone())
        })
    }
}
