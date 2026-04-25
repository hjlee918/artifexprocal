use std::io::Write;
use std::net::TcpStream;
use hal::traits::{PatternGenerator, PatternGeneratorExt};
use hal::error::PatternGenError;
use hal::types::RGB;
use crate::xml_protocol::{XmlPatch, XmlPattern};
use crate::devicecontrol_client::DeviceControlClient;
use crate::patterns_catalog::ted_disk_patterns;

pub enum PGeneratorMode {
    Direct { ip: String, port: u16 },
    DeviceControl { port: u16 },
}

pub struct PGeneratorController {
    mode: PGeneratorMode,
    connected: bool,
    stream: Option<TcpStream>,
    dc_client: Option<DeviceControlClient>,
}

impl PGeneratorController {
    pub fn direct(ip: &str, port: u16) -> Self {
        Self {
            mode: PGeneratorMode::Direct { ip: ip.to_string(), port },
            connected: false,
            stream: None,
            dc_client: None,
        }
    }

    pub fn devicecontrol(port: u16) -> Self {
        Self {
            mode: PGeneratorMode::DeviceControl { port },
            connected: false,
            stream: None,
            dc_client: Some(DeviceControlClient::new("localhost", port)),
        }
    }
}

impl PatternGenerator for PGeneratorController {
    fn connect(&mut self) -> Result<(), PatternGenError> {
        match &self.mode {
            PGeneratorMode::Direct { ip, port } => {
                let stream = TcpStream::connect(format!("{}:{}", ip, port))
                    .map_err(|e| PatternGenError::ConnectionFailed(e.to_string()))?;
                self.stream = Some(stream);
                self.connected = true;
                Ok(())
            }
            PGeneratorMode::DeviceControl { .. } => {
                if let Some(ref client) = self.dc_client {
                    client.list_patterns()?;
                }
                self.connected = true;
                Ok(())
            }
        }
    }

    fn disconnect(&mut self) {
        if let PGeneratorMode::Direct { .. } = &self.mode {
            if let Some(mut stream) = self.stream.take() {
                let black = XmlPatch::black();
                let _ = stream.write_all(black.to_xml().as_bytes());
            }
        }
        self.connected = false;
    }

    fn display_patch(&mut self, color: &RGB) -> Result<(), PatternGenError> {
        if !self.connected {
            return Err(PatternGenError::ConnectionFailed("Not connected".to_string()));
        }
        match &self.mode {
            PGeneratorMode::Direct { .. } => {
                let patch = XmlPatch {
                    r: (color.r * 255.0).round() as u8,
                    g: (color.g * 255.0).round() as u8,
                    b: (color.b * 255.0).round() as u8,
                };
                if let Some(ref mut stream) = self.stream {
                    stream.write_all(patch.to_xml().as_bytes())
                        .map_err(|e| PatternGenError::ConnectionFailed(e.to_string()))?;
                    Ok(())
                } else {
                    Err(PatternGenError::ConnectionFailed("No stream".to_string()))
                }
            }
            PGeneratorMode::DeviceControl { .. } => {
                if let Some(ref client) = self.dc_client {
                    client.display_patch(
                        (color.r * 255.0).round() as u8,
                        (color.g * 255.0).round() as u8,
                        (color.b * 255.0).round() as u8,
                    )
                } else {
                    Err(PatternGenError::ConnectionFailed("No DeviceControl client".to_string()))
                }
            }
        }
    }
}

impl PatternGeneratorExt for PGeneratorController {
    fn display_pattern(&mut self, pattern_name: &str) -> Result<(), PatternGenError> {
        if !self.connected {
            return Err(PatternGenError::ConnectionFailed("Not connected".to_string()));
        }
        if !ted_disk_patterns().contains(&pattern_name) {
            return Err(PatternGenError::ConnectionFailed(format!(
                "Pattern '{}' not in catalog", pattern_name
            )));
        }
        match &self.mode {
            PGeneratorMode::Direct { .. } => {
                let pat = XmlPattern {
                    name: pattern_name.to_string(),
                    chapter: 1,
                };
                if let Some(ref mut stream) = self.stream {
                    stream.write_all(pat.to_xml().as_bytes())
                        .map_err(|e| PatternGenError::ConnectionFailed(e.to_string()))?;
                    Ok(())
                } else {
                    Err(PatternGenError::ConnectionFailed("No stream".to_string()))
                }
            }
            PGeneratorMode::DeviceControl { .. } => {
                if let Some(ref client) = self.dc_client {
                    client.display_pattern(pattern_name)
                } else {
                    Err(PatternGenError::ConnectionFailed("No DeviceControl client".to_string()))
                }
            }
        }
    }

    fn list_patterns(&self) -> Vec<String> {
        ted_disk_patterns().iter().map(|s| s.to_string()).collect()
    }
}
