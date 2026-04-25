use reqwest::blocking::Client;
use hal::error::PatternGenError;

pub struct DeviceControlClient {
    base_url: String,
    client: Client,
}

impl DeviceControlClient {
    pub fn new(host: &str, port: u16) -> Self {
        let base_url = format!("http://{}:{}", host, port);
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub fn display_pattern(&self, name: &str) -> Result<(), PatternGenError> {
        let url = format!("{}/api/display_pattern", self.base_url);
        let resp = self.client.post(&url)
            .json(&serde_json::json!({"name": name}))
            .send()
            .map_err(|e| PatternGenError::ConnectionFailed(format!("DeviceControl: {}", e)))?;
        if !resp.status().is_success() {
            return Err(PatternGenError::ConnectionFailed(format!(
                "DeviceControl returned {}", resp.status()
            )));
        }
        Ok(())
    }

    pub fn display_patch(&self, r: u8, g: u8, b: u8) -> Result<(), PatternGenError> {
        let url = format!("{}/api/display_patch", self.base_url);
        let resp = self.client.post(&url)
            .json(&serde_json::json!({"r": r, "g": g, "b": b}))
            .send()
            .map_err(|e| PatternGenError::ConnectionFailed(format!("DeviceControl: {}", e)))?;
        if !resp.status().is_success() {
            return Err(PatternGenError::ConnectionFailed(format!(
                "DeviceControl returned {}", resp.status()
            )));
        }
        Ok(())
    }

    pub fn list_patterns(&self) -> Result<Vec<String>, PatternGenError> {
        let url = format!("{}/api/list_patterns", self.base_url);
        let resp = self.client.get(&url)
            .send()
            .map_err(|e| PatternGenError::ConnectionFailed(format!("DeviceControl: {}", e)))?;
        resp.json::<Vec<String>>()
            .map_err(|e| PatternGenError::ConnectionFailed(format!("JSON parse: {}", e)))
    }
}
