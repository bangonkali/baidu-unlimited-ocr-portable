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
