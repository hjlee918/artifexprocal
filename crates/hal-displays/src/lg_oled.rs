use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use std::net::TcpStream;
use hal::traits::DisplayController;
use hal::error::DisplayError;
use hal::types::{Lut1D, Lut3D, RGBGain};
use crate::ssap_protocol::SsapMessage;
use crate::discovery::SsdpDiscovery;
use crate::pairing::PairingState;
use crate::calibration_commands::{CalibrationMode, encode_1d_lut, encode_white_balance};
use crate::devicecontrol_client::LgDeviceControlClient;

pub enum LgOledMode {
    Direct { ip: String, port: u16 },
    DeviceControl { port: u16 },
}

pub struct LgOledController {
    mode: LgOledMode,
    connected: bool,
    paired: bool,
    ws: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    calibration: CalibrationMode,
    pairing: PairingState,
    dc_client: Option<LgDeviceControlClient>,
}

impl LgOledController {
    pub fn direct(ip: &str, port: u16) -> Self {
        Self {
            mode: LgOledMode::Direct { ip: ip.to_string(), port },
            connected: false,
            paired: false,
            ws: None,
            calibration: CalibrationMode::new(),
            pairing: PairingState::new(),
            dc_client: None,
        }
    }

    pub fn devicecontrol(port: u16) -> Self {
        Self {
            mode: LgOledMode::DeviceControl { port },
            connected: false,
            paired: false,
            ws: None,
            calibration: CalibrationMode::new(),
            pairing: PairingState::new(),
            dc_client: Some(LgDeviceControlClient::new("localhost", port)),
        }
    }

    pub fn is_direct(&self) -> bool {
        matches!(self.mode, LgOledMode::Direct { .. })
    }

    pub fn is_devicecontrol(&self) -> bool {
        matches!(self.mode, LgOledMode::DeviceControl { .. })
    }

    pub fn discover(timeout_ms: u64) -> Result<Vec<String>, DisplayError> {
        SsdpDiscovery::discover(timeout_ms)
            .map_err(|e| DisplayError::ConnectionFailed(e))
    }

    pub fn pairing_state(&self) -> &PairingState {
        &self.pairing
    }

    pub fn request_pin(&mut self) {
        self.pairing.request_pin();
    }

    pub fn submit_pin(&mut self, pin: &str) {
        self.pairing.submit_pin(pin);
        self.paired = self.pairing.is_authenticated();
    }
}

impl DisplayController for LgOledController {
    fn connect(&mut self) -> Result<(), DisplayError> {
        match &self.mode {
            LgOledMode::Direct { ip, port } => {
                let url = format!("ws://{}:{}", ip, port);
                let (ws, _) = connect(url)
                    .map_err(|e| DisplayError::ConnectionFailed(e.to_string()))?;
                self.ws = Some(ws);
                self.connected = true;
                Ok(())
            }
            LgOledMode::DeviceControl { .. } => {
                self.connected = true;
                Ok(())
            }
        }
    }

    fn disconnect(&mut self) {
        if let LgOledMode::Direct { .. } = &self.mode {
            if self.calibration.is_active() {
                let _ = self.end_calibration();
            }
            if let Some(mut ws) = self.ws.take() {
                let _ = ws.close(None);
            }
        }
        self.connected = false;
    }

    fn set_picture_mode(&mut self, mode: &str) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        match &self.mode {
            LgOledMode::Direct { .. } => {
                let msg = SsapMessage::set_picture_mode(mode);
                let json = msg.to_json()
                    .map_err(|e| DisplayError::ConnectionFailed(e.to_string()))?;
                if let Some(ref mut ws) = self.ws {
                    ws.send(Message::Text(json))
                        .map_err(|e| DisplayError::ConnectionFailed(e.to_string()))?;
                }
                Ok(())
            }
            LgOledMode::DeviceControl { .. } => {
                Ok(())
            }
        }
    }

    fn upload_1d_lut(&mut self, lut: &Lut1D) -> Result<(), DisplayError> {
        if !self.connected || !self.calibration.is_active() {
            return Err(DisplayError::ConnectionFailed("Not in calibration mode".to_string()));
        }
        match &self.mode {
            LgOledMode::Direct { .. } => {
                let pic_mode = self.calibration.pic_mode().unwrap_or("expert1");
                let data = encode_1d_lut(lut);
                let msg = SsapMessage::upload_1d_lut(pic_mode, &data);
                let json = msg.to_json()
                    .map_err(|e| DisplayError::UploadFailed(e.to_string()))?;
                if let Some(ref mut ws) = self.ws {
                    ws.send(Message::Text(json))
                        .map_err(|e| DisplayError::UploadFailed(e.to_string()))?;
                }
                Ok(())
            }
            LgOledMode::DeviceControl { .. } => {
                if let Some(ref client) = self.dc_client {
                    let pic_mode = self.calibration.pic_mode().unwrap_or("expert1");
                    client.upload_1d_lut(pic_mode, lut)
                } else {
                    Err(DisplayError::UploadFailed("No DeviceControl client".to_string()))
                }
            }
        }
    }

    fn upload_3d_lut(&mut self, _lut: &Lut3D) -> Result<(), DisplayError> {
        Err(DisplayError::UploadFailed("3D LUT upload not yet implemented".to_string()))
    }

    fn set_white_balance(&mut self, gains: RGBGain) -> Result<(), DisplayError> {
        if !self.connected || !self.calibration.is_active() {
            return Err(DisplayError::ConnectionFailed("Not in calibration mode".to_string()));
        }
        match &self.mode {
            LgOledMode::Direct { .. } => {
                let (r, g, b) = encode_white_balance(&gains);
                let msg = SsapMessage::set_white_balance(r, g, b);
                let json = msg.to_json()
                    .map_err(|e| DisplayError::UploadFailed(e.to_string()))?;
                if let Some(ref mut ws) = self.ws {
                    ws.send(Message::Text(json))
                        .map_err(|e| DisplayError::UploadFailed(e.to_string()))?;
                }
                Ok(())
            }
            LgOledMode::DeviceControl { .. } => {
                if let Some(ref client) = self.dc_client {
                    client.set_white_balance(&gains)
                } else {
                    Err(DisplayError::UploadFailed("No DeviceControl client".to_string()))
                }
            }
        }
    }
}

impl LgOledController {
    pub fn start_calibration(&mut self, pic_mode: &str) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        match &self.mode {
            LgOledMode::Direct { .. } => {
                let msg = SsapMessage::start_calibration(pic_mode);
                let json = msg.to_json()
                    .map_err(|e| DisplayError::ConnectionFailed(e.to_string()))?;
                if let Some(ref mut ws) = self.ws {
                    ws.send(Message::Text(json))
                        .map_err(|e| DisplayError::ConnectionFailed(e.to_string()))?;
                }
                self.calibration.start(pic_mode);
                Ok(())
            }
            LgOledMode::DeviceControl { .. } => {
                if let Some(ref client) = self.dc_client {
                    client.start_calibration(pic_mode)?;
                    self.calibration.start(pic_mode);
                    Ok(())
                } else {
                    Err(DisplayError::ConnectionFailed("No DeviceControl client".to_string()))
                }
            }
        }
    }

    pub fn end_calibration(&mut self) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        match &self.mode {
            LgOledMode::Direct { .. } => {
                let pic_mode = self.calibration.pic_mode().unwrap_or("expert1");
                let msg = SsapMessage::end_calibration(pic_mode);
                let json = msg.to_json()
                    .map_err(|e| DisplayError::ConnectionFailed(e.to_string()))?;
                if let Some(ref mut ws) = self.ws {
                    ws.send(Message::Text(json))
                        .map_err(|e| DisplayError::ConnectionFailed(e.to_string()))?;
                }
                self.calibration.end();
                Ok(())
            }
            LgOledMode::DeviceControl { .. } => {
                if let Some(ref client) = self.dc_client {
                    let pic_mode = self.calibration.pic_mode().unwrap_or("expert1");
                    client.end_calibration(pic_mode)?;
                    self.calibration.end();
                    Ok(())
                } else {
                    Err(DisplayError::ConnectionFailed("No DeviceControl client".to_string()))
                }
            }
        }
    }
}
