use hal::traits::Meter;
use hal::error::MeterError;
use color_science::types::XYZ;
use crate::hid_util::{HidContext, I1_PRO_2, send_command, read_response, SyncHidDevice};
use crate::commands::{CMD_GET_FIRMWARE, CMD_SET_EMISSIVE, CMD_TRIGGER_MEASURE, CMD_INITIALIZE, XriteStatus};
use crate::spectro_trait::Spectrophotometer;

pub struct I1Pro2 {
    ctx: Option<HidContext>,
    device: Option<SyncHidDevice>,
    serial: Option<String>,
    connected: bool,
}

impl I1Pro2 {
    pub fn new() -> Self {
        Self {
            ctx: None,
            device: None,
            serial: None,
            connected: false,
        }
    }

    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    pub fn initialize(&mut self) -> Result<(), MeterError> {
        let device = self.device.as_mut().ok_or_else(|| {
            MeterError::ConnectionFailed("Device not open".to_string())
        })?;

        send_command(device, CMD_INITIALIZE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response(device, 10000)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        if resp.is_empty() || !XriteStatus::from_byte(resp[0]).is_ok() {
            return Err(MeterError::ConnectionFailed(
                "Initialization failed".to_string(),
            ));
        }

        Ok(())
    }

    pub fn read_spectrum_raw(&mut self) -> Result<[f64; 36], MeterError> {
        let device = self.device.as_mut().ok_or_else(|| {
            MeterError::ConnectionFailed("Device not open".to_string())
        })?;

        send_command(device, CMD_TRIGGER_MEASURE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response(device, 8000)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        if resp.len() < 14 + 36 * 4 {
            return Err(MeterError::InvalidResponse(
                "Spectrum response too short".to_string(),
            ));
        }
        if !XriteStatus::from_byte(resp[0]).is_ok() {
            return Err(MeterError::InvalidResponse(format!(
                "Spectrum read failed: status {:02X}",
                resp[0]
            )));
        }

        let mut spectrum = [0.0f64; 36];
        for i in 0..36 {
            let offset = 14 + i * 4;
            let val = f32::from_le_bytes([
                resp[offset],
                resp[offset + 1],
                resp[offset + 2],
                resp[offset + 3],
            ]);
            spectrum[i] = val as f64;
        }
        Ok(spectrum)
    }
}

impl Meter for I1Pro2 {
    fn connect(&mut self) -> Result<(), MeterError> {
        let ctx = HidContext::new().map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let mut device = ctx
            .open_device(&I1_PRO_2)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;

        // Verify firmware
        send_command(&mut device, CMD_GET_FIRMWARE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response(&mut device, 2000)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        if resp.is_empty() || XriteStatus::from_byte(resp[0]).is_ok() {
            // Firmware response received
        }

        // Set emissive mode
        send_command(&mut device, CMD_SET_EMISSIVE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response(&mut device, 2000)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        if !resp.is_empty() && !XriteStatus::from_byte(resp[0]).is_ok() {
            return Err(MeterError::ConnectionFailed(
                "Failed to set emissive mode".to_string(),
            ));
        }

        self.ctx = Some(ctx);
        self.device = Some(device);
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.device = None;
        self.ctx = None;
        self.connected = false;
    }

    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError> {
        if !self.connected {
            return Err(MeterError::ConnectionFailed("Not connected".to_string()));
        }
        let device = self.device.as_mut().ok_or_else(|| {
            MeterError::ConnectionFailed("Device not open".to_string())
        })?;

        // Trigger measurement
        send_command(device, CMD_TRIGGER_MEASURE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response(device, integration_time_ms as i32 + 2000)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        if resp.len() < 14 {
            return Err(MeterError::InvalidResponse("Response too short".to_string()));
        }
        if !XriteStatus::from_byte(resp[0]).is_ok() {
            return Err(MeterError::InvalidResponse(format!(
                "Measurement failed: status {:02X}",
                resp[0]
            )));
        }

        let x = f32::from_le_bytes([resp[2], resp[3], resp[4], resp[5]]);
        let y = f32::from_le_bytes([resp[6], resp[7], resp[8], resp[9]]);
        let z = f32::from_le_bytes([resp[10], resp[11], resp[12], resp[13]]);

        Ok(XYZ {
            x: x as f64,
            y: y as f64,
            z: z as f64,
        })
    }

    fn model(&self) -> &str {
        "i1 Pro 2"
    }
}

impl Spectrophotometer for I1Pro2 {
    fn read_spectrum(&mut self) -> Result<[f64; 36], MeterError> {
        self.read_spectrum_raw()
    }
}
