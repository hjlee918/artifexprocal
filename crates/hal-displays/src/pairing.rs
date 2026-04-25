#[derive(Debug, Clone)]
pub enum PairingState {
    Idle,
    WaitingForPin,
    Authenticated { client_key: String },
    Failed(String),
}

impl PairingState {
    pub fn new() -> Self {
        Self::Idle
    }

    pub fn request_pin(&mut self) {
        *self = Self::WaitingForPin;
    }

    pub fn submit_pin(&mut self, pin: &str) {
        *self = Self::Authenticated {
            client_key: format!("key_{}", pin),
        };
    }

    pub fn fail(&mut self, reason: String) {
        *self = Self::Failed(reason);
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn is_waiting_for_pin(&self) -> bool {
        matches!(self, Self::WaitingForPin)
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, Self::Authenticated { .. })
    }

    pub fn client_key(&self) -> Option<&str> {
        match self {
            Self::Authenticated { client_key } => Some(client_key),
            _ => None,
        }
    }
}

pub struct PairingMessage;

impl PairingMessage {
    pub fn request_key(app_name: &str) -> String {
        format!(
            r#"{{"type":"request","uri":"ssap://com.webos.service.tvpairing/getKey","payload":{{"clientName":"{}"}}}}"#,
            app_name
        )
    }

    pub fn send_pin(pin: &str) -> String {
        format!(
            r#"{{"type":"request","uri":"ssap://com.webos.service.tvpairing/sendKey","payload":{{"key":"{}"}}}}"#,
            pin
        )
    }
}
