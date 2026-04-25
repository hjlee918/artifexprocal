use reqwest::blocking::Client;
use hal::error::DisplayError;
use hal::types::{Lut1D, RGBGain};
use crate::calibration_commands::{encode_1d_lut, encode_white_balance};

pub struct LgDeviceControlClient {
    base_url: String,
    client: Client,
}

impl LgDeviceControlClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: format!("http://{}:{}", host, port),
            client: Client::new(),
        }
    }

    pub fn connect(&self, tv_ip: &str, tv_port: u16) -> Result<(), DisplayError> {
        let url = format!("{}/api/lg/connect", self.base_url);
        let resp = self.client.post(&url)
            .json(&serde_json::json!({"ip": tv_ip, "port": tv_port}))
            .send()
            .map_err(|e| DisplayError::ConnectionFailed(format!("DeviceControl: {}", e)))?;
        if !resp.status().is_success() {
            return Err(DisplayError::ConnectionFailed(format!(
                "DeviceControl returned {}", resp.status()
            )));
        }
        Ok(())
    }

    pub fn start_calibration(&self, pic_mode: &str) -> Result<(), DisplayError> {
        let url = format!("{}/api/lg/start_calibration", self.base_url);
        let resp = self.client.post(&url)
            .json(&serde_json::json!({"picMode": pic_mode}))
            .send()
            .map_err(|e| DisplayError::ConnectionFailed(format!("DeviceControl: {}", e)))?;
        if !resp.status().is_success() {
            return Err(DisplayError::ConnectionFailed(format!(
                "DeviceControl returned {}", resp.status()
            )));
        }
        Ok(())
    }

    pub fn upload_1d_lut(&self, pic_mode: &str, lut: &Lut1D) -> Result<(), DisplayError> {
        let url = format!("{}/api/lg/upload_1d_lut", self.base_url);
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let data = encode_1d_lut(lut);
        let resp = self.client.post(&url)
            .json(&serde_json::json!({
                "picMode": pic_mode,
                "data": STANDARD.encode(&data),
            }))
            .send()
            .map_err(|e| DisplayError::UploadFailed(format!("DeviceControl: {}", e)))?;
        if !resp.status().is_success() {
            return Err(DisplayError::UploadFailed(format!(
                "DeviceControl returned {}", resp.status()
            )));
        }
        Ok(())
    }

    pub fn set_white_balance(&self, gains: &RGBGain) -> Result<(), DisplayError> {
        let url = format!("{}/api/lg/set_white_balance", self.base_url);
        let (r, g, b) = encode_white_balance(gains);
        let resp = self.client.post(&url)
            .json(&serde_json::json!({"rGain": r, "gGain": g, "bGain": b}))
            .send()
            .map_err(|e| DisplayError::UploadFailed(format!("DeviceControl: {}", e)))?;
        if !resp.status().is_success() {
            return Err(DisplayError::UploadFailed(format!(
                "DeviceControl returned {}", resp.status()
            )));
        }
        Ok(())
    }

    pub fn end_calibration(&self, pic_mode: &str) -> Result<(), DisplayError> {
        let url = format!("{}/api/lg/end_calibration", self.base_url);
        let resp = self.client.post(&url)
            .json(&serde_json::json!({"picMode": pic_mode}))
            .send()
            .map_err(|e| DisplayError::ConnectionFailed(format!("DeviceControl: {}", e)))?;
        if !resp.status().is_success() {
            return Err(DisplayError::ConnectionFailed(format!(
                "DeviceControl returned {}", resp.status()
            )));
        }
        Ok(())
    }

    pub fn connect_url(host: &str, port: u16, tv_ip: &str, tv_port: u16) -> String {
        format!("http://{}:{}/api/lg/connect?ip={}&port={}", host, port, tv_ip, tv_port)
    }

    pub fn start_calibration_url(host: &str, port: u16, pic_mode: &str) -> String {
        format!("http://{}:{}/api/lg/start_calibration?picMode={}", host, port, pic_mode)
    }
}
