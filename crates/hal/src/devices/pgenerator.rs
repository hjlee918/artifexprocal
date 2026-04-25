use crate::error::PatternGenError;
use crate::traits::PatternGenerator;
use color_science::types::RGB;

pub struct PGenerator {
    ip: String,
    connected: bool,
}

impl PGenerator {
    pub fn new(ip: &str) -> Self {
        Self {
            ip: ip.to_string(),
            connected: false,
        }
    }
}

impl PatternGenerator for PGenerator {
    fn connect(&mut self) -> Result<(), PatternGenError> {
        if !is_valid_ip(&self.ip) {
            return Err(PatternGenError::ConnectionFailed(
                format!("Invalid IP address: {}", self.ip)
            ));
        }
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
    }

    fn display_patch(&mut self, _color: &RGB) -> Result<(), PatternGenError> {
        if !self.connected {
            return Err(PatternGenError::ConnectionFailed("Not connected".to_string()));
        }
        Ok(())
    }
}

fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}
