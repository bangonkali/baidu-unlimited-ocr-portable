use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

use crate::{
    realtime_event_types::supported_event_types,
    storage::{Repository, StoredRealtimeEvent},
};

const RECENT_OCR_EVENT_LIMIT: usize = 50_000;
const REALTIME_BROADCAST_CHANNEL_LIMIT: usize = 16_384;
const REALTIME_PERSIST_QUEUE_LIMIT: usize = 16_384;
const REALTIME_PERSIST_BATCH_LIMIT: usize = 256;
const REALTIME_PERSIST_FLUSH_MS: u64 = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventEnvelope {
    pub(crate) version: u32,
    pub(crate) sequence: u64,
    #[serde(rename = "type")]
    pub(crate) event_type: String,
    pub(crate) occurred_at: String,
    pub(crate) payload: Value,
}

#[derive(Debug)]
pub(crate) struct RealtimeHub {
    sequence: AtomicU64,
    sender: broadcast::Sender<EventEnvelope>,
    persist_sender: RwLock<Option<mpsc::Sender<StoredRealtimeEvent>>>,
    persist_worker: Mutex<Option<JoinHandle<()>>>,
    recent_ocr_events: Mutex<VecDeque<StoredRealtimeEvent>>,
}

impl RealtimeHub {
    #[must_use]
    pub(crate) fn new() -> Arc<Self> {
        let (sender, _) = broadcast::channel(REALTIME_BROADCAST_CHANNEL_LIMIT);
        Arc::new(Self {
            sequence: AtomicU64::new(0),
            sender,
            persist_sender: RwLock::new(None),
            persist_worker: Mutex::new(None),
            recent_ocr_events: Mutex::new(VecDeque::with_capacity(RECENT_OCR_EVENT_LIMIT)),
        })
    }

    pub(crate) fn attach_repository(&self, repository: Repository) {
        let (sender, receiver) = mpsc::channel(REALTIME_PERSIST_QUEUE_LIMIT);
        if let Ok(mut guard) = self.persist_sender.write() {
            *guard = Some(sender);
        }
        let handle = tokio::spawn(realtime_persist_worker(repository, receiver));
        if let Ok(mut guard) = self.persist_worker.lock()
            && let Some(previous) = guard.replace(handle)
        {
            previous.abort();
        }
    }

    pub(crate) async fn shutdown_persistence(&self, duration: Duration) {
        let sender = self
            .persist_sender
            .write()
            .ok()
            .and_then(|mut guard| guard.take());
        drop(sender);
        let handle = self
            .persist_worker
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());
        let Some(mut handle) = handle else {
            return;
        };
        let timeout = tokio::time::sleep(duration);
        tokio::pin!(timeout);
        tokio::select! {
            result = &mut handle => {
                if let Err(error) = result {
                    tracing::warn!(%error, "realtime persistence worker failed during shutdown");
                }
            }
            () = &mut timeout => {
                handle.abort();
                tracing::warn!("realtime persistence worker did not drain before shutdown timeout");
            }
        }
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }

    pub(crate) fn last_sequence(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }

    pub(crate) fn publish(&self, event_type: &str, payload: Value) -> EventEnvelope {
        let envelope = EventEnvelope {
            version: 1,
            sequence: self.sequence.fetch_add(1, Ordering::SeqCst) + 1,
            event_type: event_type.to_string(),
            occurred_at: Utc::now().to_rfc3339(),
            payload,
        };
        if let Some(event) = stored_realtime_event(&envelope) {
            self.remember_recent_event(event.clone());
            let persist_sender = self
                .persist_sender
                .read()
                .ok()
                .and_then(|guard| guard.as_ref().cloned());
            if let Some(sender) = persist_sender
                && let Err(error) = sender.try_send(event)
            {
                tracing::warn!(%error, "failed to queue realtime event persistence");
            }
        }
        let _ = self.sender.send(envelope.clone());
        envelope
    }

    pub(crate) fn recent_ocr_events(
        &self,
        run_id: Option<&str>,
        file_hash: Option<&str>,
        page_no: Option<u32>,
        since_sequence: Option<u64>,
        limit: usize,
    ) -> Vec<StoredRealtimeEvent> {
        let since_sequence = since_sequence.unwrap_or(0);
        let Ok(events) = self.recent_ocr_events.lock() else {
            return Vec::new();
        };
        events
            .iter()
            .filter(|event| event.sequence > since_sequence)
            .filter(|event| run_id.is_none_or(|value| event.run_id.as_deref() == Some(value)))
            .filter(|event| file_hash.is_none_or(|value| event.file_hash.as_deref() == Some(value)))
            .filter(|event| page_no.is_none_or(|value| event.page_no == Some(value)))
            .take(limit)
            .cloned()
            .collect()
    }

    fn remember_recent_event(&self, event: StoredRealtimeEvent) {
        let Ok(mut events) = self.recent_ocr_events.lock() else {
            return;
        };
        if events.len() >= RECENT_OCR_EVENT_LIMIT {
            events.pop_front();
        }
        events.push_back(event);
    }

    pub(crate) fn ready_payload(&self) -> Value {
        json!({
            "path": "/api/events",
            "heartbeat": "native-websocket",
            "last_sequence": self.last_sequence(),
            "supported_types": supported_event_types(),
        })
    }

    fn ready_envelope(&self) -> EventEnvelope {
        EventEnvelope {
            version: 1,
            sequence: self.last_sequence(),
            event_type: "connection.ready".to_string(),
            occurred_at: Utc::now().to_rfc3339(),
            payload: self.ready_payload(),
        }
    }
}

async fn realtime_persist_worker(
    repository: Repository,
    mut receiver: mpsc::Receiver<StoredRealtimeEvent>,
) {
    let mut batch = Vec::with_capacity(REALTIME_PERSIST_BATCH_LIMIT);
    let mut flush_interval =
        tokio::time::interval(Duration::from_millis(REALTIME_PERSIST_FLUSH_MS));
    flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            event = receiver.recv() => {
                let Some(event) = event else {
                    flush_realtime_batch(&repository, &mut batch).await;
                    break;
                };
                batch.push(event);
                if batch.len() >= REALTIME_PERSIST_BATCH_LIMIT {
                    flush_realtime_batch(&repository, &mut batch).await;
                }
            }
            _ = flush_interval.tick() => {
                flush_realtime_batch(&repository, &mut batch).await;
            }
        }
    }
}

async fn flush_realtime_batch(repository: &Repository, batch: &mut Vec<StoredRealtimeEvent>) {
    if batch.is_empty() {
        return;
    }
    let events = std::mem::take(batch);
    if let Err(error) = repository.persist_realtime_events(events).await {
        tracing::warn!(%error, "failed to persist realtime event batch");
    }
}

fn stored_realtime_event(envelope: &EventEnvelope) -> Option<StoredRealtimeEvent> {
    if !envelope.event_type.starts_with("ocr.page.") {
        return None;
    }
    Some(StoredRealtimeEvent {
        event_id: crate::ids::new_persistence_id(),
        sequence: envelope.sequence,
        event_type: envelope.event_type.clone(),
        occurred_at: envelope.occurred_at.clone(),
        run_id: envelope
            .payload
            .get("run_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        file_hash: envelope
            .payload
            .get("file_hash")
            .and_then(Value::as_str)
            .map(str::to_string),
        page_no: envelope
            .payload
            .get("page_no")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok()),
        payload: envelope.payload.clone(),
    })
}

pub(crate) async fn websocket(mut socket: WebSocket, hub: Arc<RealtimeHub>) {
    let mut receiver = hub.subscribe();
    let ready = hub.ready_envelope();
    if send_json(&mut socket, &ready).await.is_err() {
        return;
    }
    loop {
        tokio::select! {
            message = socket.recv() => {
                if message.is_none() {
                    break;
                }
            }
            event = receiver.recv() => {
                match event {
                    Ok(event) => {
                        if send_json(&mut socket, &event).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "websocket realtime receiver lagged");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn send_json(socket: &mut WebSocket, event: &EventEnvelope) -> Result<(), axum::Error> {
    let Ok(payload) = serde_json::to_string(event) else {
        return Ok(());
    };
    socket.send(Message::Text(payload.into())).await
}

#[cfg(test)]
#[path = "realtime_tests.rs"]
mod tests;
