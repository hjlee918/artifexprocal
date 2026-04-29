use crate::error::MeterError;
use crate::traits::Meter;
use color_science::types::XYZ;

pub struct I1DisplayPro {
    _path: String,
    connected: bool,
}

impl I1DisplayPro {
    pub fn new(path: &str) -> Self {
        Self {
            _path: path.to_string(),
            connected: false,
        }
    }
}

impl Meter for I1DisplayPro {
    fn connect(&mut self) -> Result<(), MeterError> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
    }

    fn read_xyz(&mut self, _integration_time_ms: u32) -> Result<XYZ, MeterError> {
        if !self.connected {
            return Err(MeterError::ConnectionFailed("Not connected".to_string()));
        }
        Ok(XYZ { x: 95.047, y: 100.0, z: 108.883 })
    }

    fn model(&self) -> &str {
        "i1 Display Pro Rev.B"
    }
}
