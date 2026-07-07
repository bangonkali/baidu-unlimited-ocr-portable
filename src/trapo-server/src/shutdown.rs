use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Utc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub(crate) struct ShutdownCoordinator {
    inner: Arc<ShutdownInner>,
}

#[derive(Debug)]
struct ShutdownInner {
    record: Mutex<Option<ShutdownRecord>>,
    token: CancellationToken,
}

#[derive(Debug, Clone)]
pub(crate) struct ShutdownRecord {
    pub(crate) requested_at: String,
    pub(crate) source: String,
}

impl ShutdownCoordinator {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(ShutdownInner {
                record: Mutex::new(None),
                token: CancellationToken::new(),
            }),
        }
    }

    #[must_use]
    pub(crate) fn is_requested(&self) -> bool {
        self.inner
            .record
            .lock()
            .is_ok_and(|record| record.is_some())
    }

    #[must_use]
    pub(crate) fn record(&self) -> Option<ShutdownRecord> {
        self.inner
            .record
            .lock()
            .ok()
            .and_then(|record| record.clone())
    }

    pub(crate) fn request(&self, source: &str) -> ShutdownRecord {
        // skylos: ignore[unused_functions] called through AppState shutdown include from route and signal handlers.
        let requested = ShutdownRecord {
            requested_at: Utc::now().to_rfc3339(),
            source: source.to_string(),
        };
        let record = match self.inner.record.lock() {
            Ok(mut guard) => guard.get_or_insert_with(|| requested).clone(),
            Err(_) => requested,
        };
        self.inner.token.cancel();
        record
    }

    #[must_use]
    pub(crate) fn token(&self) -> CancellationToken {
        // skylos: ignore[unused_functions] called through AppState shutdown include from the Axum graceful shutdown future.
        self.inner.token.clone()
    }
}

#[derive(Debug, Default)]
pub(crate) struct BackgroundTasks {
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl BackgroundTasks {
    pub(crate) fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        if let Ok(mut handles) = self.handles.lock() {
            handles.push(handle);
        } else {
            handle.abort();
        }
    }

    pub(crate) async fn wait_or_abort(&self, duration: Duration) -> usize {
        let mut handles = self.take_handles();
        let timeout = tokio::time::sleep(duration);
        tokio::pin!(timeout);
        loop {
            await_finished(&mut handles).await;
            if handles.is_empty() {
                return 0;
            }
            tokio::select! {
                () = &mut timeout => break,
                () = tokio::time::sleep(Duration::from_millis(25)) => {}
            }
        }
        let remaining = handles.len();
        for handle in handles {
            handle.abort();
        }
        remaining
    }

    fn take_handles(&self) -> Vec<JoinHandle<()>> {
        self.handles
            .lock()
            .map(|mut handles| std::mem::take(&mut *handles))
            .unwrap_or_default()
    }
}

async fn await_finished(handles: &mut Vec<JoinHandle<()>>) {
    let mut index = 0;
    while index < handles.len() {
        if handles[index].is_finished() {
            let handle = handles.swap_remove(index);
            let _ = handle.await;
        } else {
            index += 1;
        }
    }
}
