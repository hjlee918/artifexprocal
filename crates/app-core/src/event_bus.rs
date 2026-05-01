//! Event bus for backend-to-frontend communication.
//!
//! Uses tokio::sync::broadcast for fan-out event delivery.

use tokio::sync::broadcast;

/// Default channel capacity for the event bus.
const DEFAULT_CAPACITY: usize = 256;

/// Events emitted by modules and the application core.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleEvent {
    /// A module-specific event with arbitrary JSON payload.
    ModuleEvent {
        module_id: String,
        event_type: String,
        payload: serde_json::Value,
    },
    /// A command completed successfully.
    CommandCompleted {
        module_id: String,
        command: String,
    },
    /// An error occurred in a module.
    Error {
        source: String,
        message: String,
    },
}

/// Fan-out event bus using tokio broadcast channels.
pub struct EventBus {
    sender: broadcast::Sender<ModuleEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Create a new event bus.
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(DEFAULT_CAPACITY);
        Self { sender }
    }

    /// Subscribe to events. Returns a receiver that will receive all
    /// events published after subscription.
    pub fn subscribe(&self) -> broadcast::Receiver<ModuleEvent> {
        self.sender.subscribe()
    }

    /// Publish an event to all subscribers.
    pub fn publish(&self, event: ModuleEvent) {
        // Ignore send errors (happens when no receivers are connected).
        let _ = self.sender.send(event);
    }

    /// Number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn event_bus_delivers_to_subscribers() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let event = ModuleEvent::CommandCompleted {
            module_id: "test".to_string(),
            command: "ping".to_string(),
        };
        bus.publish(event.clone());

        let received = rx.recv().await.expect("should receive event");
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn event_bus_fanout() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let event = ModuleEvent::Error {
            source: "test".to_string(),
            message: "boom".to_string(),
        };
        bus.publish(event.clone());

        let r1 = rx1.recv().await.expect("rx1 should receive");
        let r2 = rx2.recv().await.expect("rx2 should receive");
        assert_eq!(r1, event);
        assert_eq!(r2, event);
    }
}
