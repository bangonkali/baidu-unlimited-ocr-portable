const ANNOTATION_PERSIST_QUEUE_LIMIT: usize = 8_192;
const ANNOTATION_PERSIST_BATCH_LIMIT: usize = 512;
const ANNOTATION_PERSIST_FLUSH_MS: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AnnotationIdentityKey {
    run_id: String,
    file_hash: String,
    page_no: u32,
    source_region_key: String,
}

#[derive(Debug, Clone)]
struct ResolvedAnnotationIdentityDraft {
    annotation_id: String,
    draft: AnnotationIdentityDraft,
}

#[derive(Debug, Clone)]
struct AnnotationIdentityRuntime {
    cache: Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, String>>>,
    pending: Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, AnnotationIdentityDraft>>>,
    signal: Arc<std::sync::Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
    worker: Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl AnnotationIdentityRuntime {
    fn new(repository: Repository) -> Self {
        let (signal, receiver) = tokio::sync::mpsc::channel(ANNOTATION_PERSIST_QUEUE_LIMIT);
        let runtime = Self {
            cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            pending: Arc::new(std::sync::Mutex::new(HashMap::new())),
            signal: Arc::new(std::sync::Mutex::new(Some(signal))),
            worker: Arc::new(std::sync::Mutex::new(None)),
        };
        let worker = tokio::spawn(annotation_identity_worker(
            repository,
            runtime.pending.clone(),
            receiver,
        ));
        if let Ok(mut guard) = runtime.worker.lock() {
            *guard = Some(worker);
        }
        runtime
    }

    #[cfg(test)]
    fn new_for_tests() -> Self {
        let (signal, _receiver) = tokio::sync::mpsc::channel(ANNOTATION_PERSIST_QUEUE_LIMIT);
        Self {
            cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            pending: Arc::new(std::sync::Mutex::new(HashMap::new())),
            signal: Arc::new(std::sync::Mutex::new(Some(signal))),
            worker: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn resolve_and_enqueue(
        &self,
        draft: AnnotationIdentityDraft,
    ) -> ResolvedAnnotationIdentityDraft {
        let key = AnnotationIdentityKey::from_draft(&draft);
        let annotation_id = self.annotation_id_for(&key);
        let mut resolved = draft;
        resolved.annotation_id = Some(annotation_id.clone());
        self.enqueue(key, resolved.clone());
        ResolvedAnnotationIdentityDraft {
            annotation_id,
            draft: resolved,
        }
    }

    async fn persist_now(
        &self,
        repository: &Repository,
        drafts: &[AnnotationIdentityDraft],
    ) -> Result<()> {
        repository
            .persist_discovered_annotations(drafts.to_vec())
            .await?;
        self.mark_persisted(drafts);
        Ok(())
    }

    fn annotation_id_for(&self, key: &AnnotationIdentityKey) -> String {
        let Ok(mut cache) = self.cache.lock() else {
            tracing::warn!("annotation identity cache mutex poisoned");
            return new_persistence_id();
        };
        cache
            .entry(key.clone())
            .or_insert_with(new_persistence_id)
            .clone()
    }

    fn enqueue(&self, key: AnnotationIdentityKey, draft: AnnotationIdentityDraft) {
        let Ok(mut pending) = self.pending.lock() else {
            tracing::warn!("annotation identity pending mutex poisoned");
            return;
        };
        pending.insert(key, draft);
        drop(pending);
        let signal = self
            .signal
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned());
        let Some(signal) = signal else {
            return;
        };
        if let Err(error) = signal.try_send(()) {
            tracing::debug!(
                %error,
                "annotation identity persistence signal skipped; page completion will flush"
            );
        }
    }

    async fn shutdown(&self, repository: &Repository, duration: std::time::Duration) {
        flush_all_annotation_batches(repository, &self.pending).await;
        let signal = self.signal.lock().ok().and_then(|mut guard| guard.take());
        drop(signal);
        let handle = self.worker.lock().ok().and_then(|mut guard| guard.take());
        let Some(mut handle) = handle else {
            return;
        };
        let timeout = tokio::time::sleep(duration);
        tokio::pin!(timeout);
        tokio::select! {
            result = &mut handle => {
                if let Err(error) = result {
                    tracing::warn!(%error, "annotation identity worker failed during shutdown");
                }
            }
            () = &mut timeout => {
                handle.abort();
                tracing::warn!("annotation identity worker did not drain before shutdown timeout");
            }
        }
    }

    fn mark_persisted(&self, drafts: &[AnnotationIdentityDraft]) {
        let Ok(mut pending) = self.pending.lock() else {
            tracing::warn!("annotation identity pending mutex poisoned");
            return;
        };
        for draft in drafts {
            let key = AnnotationIdentityKey::from_draft(draft);
            if pending
                .get(&key)
                .is_some_and(|queued| queued.annotation_id == draft.annotation_id)
            {
                pending.remove(&key);
            }
        }
    }
}

impl AnnotationIdentityKey {
    fn from_draft(draft: &AnnotationIdentityDraft) -> Self {
        Self {
            run_id: draft.run_id.clone(),
            file_hash: draft.file_hash.clone(),
            page_no: draft.page_no,
            source_region_key: draft.source_region_key.clone(),
        }
    }
}

async fn annotation_identity_worker(
    repository: Repository,
    pending: Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, AnnotationIdentityDraft>>>,
    mut receiver: tokio::sync::mpsc::Receiver<()>,
) {
    let mut flush_interval =
        tokio::time::interval(std::time::Duration::from_millis(ANNOTATION_PERSIST_FLUSH_MS));
    flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            signal = receiver.recv() => {
                if signal.is_none() {
                    flush_all_annotation_batches(&repository, &pending).await;
                    break;
                }
                flush_annotation_batch(&repository, &pending).await;
            }
            _ = flush_interval.tick() => {
                flush_annotation_batch(&repository, &pending).await;
            }
        }
    }
}

async fn flush_all_annotation_batches(
    repository: &Repository,
    pending: &Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, AnnotationIdentityDraft>>>,
) {
    while !pending_is_empty(pending) {
        flush_annotation_batch(repository, pending).await;
    }
}

async fn flush_annotation_batch(
    repository: &Repository,
    pending: &Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, AnnotationIdentityDraft>>>,
) {
    let drafts = take_pending_annotation_batch(pending);
    if drafts.is_empty() {
        return;
    }
    if let Err(error) = repository.persist_discovered_annotations(drafts.clone()).await {
        requeue_annotation_batch(pending, drafts);
        tracing::warn!(%error, "failed to persist annotation identity batch");
    }
}

fn take_pending_annotation_batch(
    pending: &Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, AnnotationIdentityDraft>>>,
) -> Vec<AnnotationIdentityDraft> {
    let Ok(mut pending) = pending.lock() else {
        tracing::warn!("annotation identity pending mutex poisoned");
        return Vec::new();
    };
    let mut remaining = ANNOTATION_PERSIST_BATCH_LIMIT;
    let mut drafts = Vec::with_capacity(remaining.min(pending.len()));
    pending.retain(|_, draft| {
        if remaining == 0 {
            return true;
        }
        remaining -= 1;
        drafts.push(draft.clone());
        false
    });
    drafts
}

fn requeue_annotation_batch(
    pending: &Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, AnnotationIdentityDraft>>>,
    drafts: Vec<AnnotationIdentityDraft>,
) {
    let Ok(mut pending) = pending.lock() else {
        tracing::warn!("annotation identity pending mutex poisoned");
        return;
    };
    for draft in drafts {
        pending
            .entry(AnnotationIdentityKey::from_draft(&draft))
            .or_insert(draft);
    }
}

fn pending_is_empty(
    pending: &Arc<std::sync::Mutex<HashMap<AnnotationIdentityKey, AnnotationIdentityDraft>>>,
) -> bool {
    pending.lock().map_or(true, |pending| pending.is_empty())
}
