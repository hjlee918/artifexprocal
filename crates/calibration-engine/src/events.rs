use calibration_core::state::CalibrationEvent;
use tokio::sync::broadcast;

pub struct EventChannel {
    sender: broadcast::Sender<CalibrationEvent>,
}

impl EventChannel {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CalibrationEvent> {
        self.sender.subscribe()
    }

    pub fn send(&self, event: CalibrationEvent) {
        let _ = self.sender.send(event);
    }
}
