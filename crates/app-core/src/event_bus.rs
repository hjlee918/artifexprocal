//! Event bus for backend-to-frontend communication.
//!
//! Uses tokio::sync::broadcast for fan-out event delivery.

use color_science::measurement::MeasurementResult;
use crate::registers::RegisterSlot;
use tokio::sync::broadcast;

/// Default channel capacity for the event bus.
const DEFAULT_CAPACITY: usize = 256;

/// Reason why a continuous read loop stopped.
#[derive(Debug, Clone, PartialEq)]
pub enum ContinuousReadStopReason {
    Cancelled,
    ErrorToleranceExceeded,
    SequenceExhausted,
    FatalError(String),
}

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
    /// A single measurement was received (from read or continuous stream).
    MeasurementReceived {
        meter_id: String,
        measurement: MeasurementResult,
    },
    /// A transient error occurred during continuous read.
    ContinuousReadError {
        meter_id: String,
        error: String,
        consecutive_count: u32,
    },
    /// The continuous read loop stopped.
    ContinuousReadStopped {
        meter_id: String,
        reason: ContinuousReadStopReason,
    },
    /// A register slot was updated by an explicit set or clear.
    RegisterChanged {
        slot: RegisterSlot,
        measurement: Option<MeasurementResult>,
    },
    /// Meter configuration was changed.
    ConfigChanged {
        meter_id: String,
        config: hal::meter::MeterConfig,
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

    #[tokio::test]
    async fn event_bus_delivers_measurement_received() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let measurement = MeasurementResult::from_xyz(
            color_science::types::Xyz {
                x: 95.047,
                y: 100.0,
                z: 108.883,
            },
            "meter-1",
            "fake-1",
            "FakeMeter",
        );
        let event = ModuleEvent::MeasurementReceived {
            meter_id: "meter-1".to_string(),
            measurement,
        };
        bus.publish(event.clone());

        let received = rx.recv().await.expect("should receive event");
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn event_bus_delivers_register_changed() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let measurement = MeasurementResult::from_xyz(
            color_science::types::Xyz {
                x: 95.047,
                y: 100.0,
                z: 108.883,
            },
            "meter-1",
            "fake-1",
            "FakeMeter",
        );
        let event = ModuleEvent::RegisterChanged {
            slot: RegisterSlot::Reference,
            measurement: Some(measurement),
        };
        bus.publish(event.clone());

        let received = rx.recv().await.expect("should receive event");
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn event_bus_delivers_config_changed() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let event = ModuleEvent::ConfigChanged {
            meter_id: "meter-1".to_string(),
            config: hal::meter::MeterConfig {
                mode: hal::meter::MeasurementMode::Ambient,
                averaging_count: 3,
                integration_time_ms: Some(250),
            },
        };
        bus.publish(event.clone());

        let received = rx.recv().await.expect("should receive event");
        assert_eq!(received, event);
    }
}
