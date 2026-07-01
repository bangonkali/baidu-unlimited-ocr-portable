use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::broadcast;

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
}

impl RealtimeHub {
    pub fn new() -> Arc<Self> {
        let (sender, _) = broadcast::channel(512);
        Arc::new(Self {
            sequence: AtomicU64::new(0),
            sender,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event_type: &str, payload: Value) -> EventEnvelope {
        let envelope = EventEnvelope {
            version: 1,
            sequence: self.sequence.fetch_add(1, Ordering::SeqCst) + 1,
            event_type: event_type.to_string(),
            occurred_at: Utc::now().to_rfc3339(),
            payload,
        };
        let _ = self.sender.send(envelope.clone());
        envelope
    }

    pub fn ready_payload() -> Value {
        json!({
            "path": "/api/events",
            "heartbeat": "native-websocket",
            "supported_types": supported_event_types(),
        })
    }
}

pub async fn websocket(mut socket: WebSocket, hub: Arc<RealtimeHub>) {
    let ready = hub.publish("connection.ready", RealtimeHub::ready_payload());
    if send_json(&mut socket, &ready).await.is_err() {
        return;
    }
    let mut receiver = hub.subscribe();
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
