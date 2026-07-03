use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};

use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::broadcast;

use crate::storage::Repository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub version: u32,
    pub sequence: u64,
    #[serde(rename = "type")]
    pub event_type: String,
    pub occurred_at: String,
    pub payload: Value,
}

#[derive(Debug)]
pub struct RealtimeHub {
    sequence: AtomicU64,
    sender: broadcast::Sender<EventEnvelope>,
    repository: RwLock<Option<Repository>>,
}

impl RealtimeHub {
    pub fn new() -> Arc<Self> {
        let (sender, _) = broadcast::channel(512);
        Arc::new(Self {
            sequence: AtomicU64::new(0),
            sender,
            repository: RwLock::new(None),
        })
    }

    pub fn attach_repository(&self, repository: Repository) {
        if let Ok(mut guard) = self.repository.write() {
            *guard = Some(repository);
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }

    pub fn last_sequence(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }

    pub fn publish(&self, event_type: &str, payload: Value) -> EventEnvelope {
        let envelope = EventEnvelope {
            version: 1,
            sequence: self.sequence.fetch_add(1, Ordering::SeqCst) + 1,
            event_type: event_type.to_string(),
            occurred_at: Utc::now().to_rfc3339(),
            payload,
        };
        let repository = self
            .repository
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().cloned());
        if let Some(repository) = repository {
            let _ = repository.persist_realtime_event(
                envelope.sequence,
                &envelope.event_type,
                &envelope.occurred_at,
                &envelope.payload,
            );
        }
        let _ = self.sender.send(envelope.clone());
        envelope
    }

    pub fn ready_payload(&self) -> Value {
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

pub async fn websocket(mut socket: WebSocket, hub: Arc<RealtimeHub>) {
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
                let Ok(event) = event else {
                    continue;
                };
                if send_json(&mut socket, &event).await.is_err() {
                    break;
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

fn supported_event_types() -> Vec<&'static str> {
    vec![
        "connection.ready",
        "status.changed",
        "model.changed",
        "run.changed",
        "document.changed",
        "document.page.changed",
        "document.regions.changed",
        "document.text.changed",
        "ocr.page.stream.started",
        "ocr.page.raw.delta",
        "ocr.page.text.patch",
        "ocr.page.region.upsert",
        "ocr.page.region.remove",
        "ocr.page.span.upsert",
        "ocr.page.span.remove",
        "ocr.page.metrics.changed",
        "ocr.page.stream.completed",
        "ocr.page.stream.failed",
        "log.appended",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast::error::TryRecvError;

    #[test]
    fn ready_envelope_does_not_broadcast_or_increment_sequence() {
        let hub = RealtimeHub::new();
        let mut receiver = hub.subscribe();

        let ready = hub.ready_envelope();

        assert_eq!(ready.event_type, "connection.ready");
        assert_eq!(ready.sequence, 0);
        assert_eq!(hub.last_sequence(), 0);
        assert!(matches!(receiver.try_recv(), Err(TryRecvError::Empty)));

        hub.publish("status.changed", json!({ "state": "running" }));
        let event = receiver.try_recv();
        assert!(event.is_ok(), "status event should broadcast: {event:?}");
        if let Ok(event) = event {
            assert_eq!(event.sequence, 1);
            assert_eq!(event.event_type, "status.changed");
        }
    }
}
