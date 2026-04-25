use crate::devices::util::is_valid_ip;
use crate::error::DisplayError;
use crate::traits::DisplayController;
use crate::types::{Lut1D, Lut3D, RGBGain};

pub struct LgOledController {
    ip: String,
    connected: bool,
}

impl LgOledController {
    pub fn new(ip: &str) -> Self {
        Self {
            ip: ip.to_string(),
            connected: false,
        }
    }
}

impl DisplayController for LgOledController {
    fn connect(&mut self) -> Result<(), DisplayError> {
        if !is_valid_ip(&self.ip) {
            return Err(DisplayError::ConnectionFailed(format!(
                "Invalid IP address: {}",
                self.ip
            )));
        }
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
    }

    fn set_picture_mode(&mut self, _mode: &str) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        Ok(())
    }

    fn upload_1d_lut(&mut self, _lut: &Lut1D) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        Ok(())
    }

    fn upload_3d_lut(&mut self, _lut: &Lut3D) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        Ok(())
    }

    fn set_white_balance(&mut self, _gains: RGBGain) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        Ok(())
    }
}
