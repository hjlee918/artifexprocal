use hal::traits::Meter;
use hal::error::MeterError;
use color_science::types::XYZ;
use crate::hid_util::{HidContext, I1_DISPLAY_PRO, send_command, read_response, SyncHidDevice};
use crate::commands::{CMD_GET_FIRMWARE, CMD_SET_EMISSIVE, CMD_TRIGGER_MEASURE, CMD_SET_INTEGRATION_TIME, XriteStatus};

pub struct I1DisplayPro {
    ctx: Option<HidContext>,
    device: Option<SyncHidDevice>,
    serial: Option<String>,
    integration_time_ms: u32,
    connected: bool,
}

impl I1DisplayPro {
    pub fn new() -> Self {
        Self {
            ctx: None,
            device: None,
            serial: None,
            integration_time_ms: 200,
            connected: false,
        }
    }

    pub fn integration_time_ms(&self) -> u32 {
        self.integration_time_ms
    }

    pub fn set_integration_time(&mut self, ms: u32) {
        self.integration_time_ms = ms.clamp(80, 5000);
    }

    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }
}

impl Meter for I1DisplayPro {
    fn connect(&mut self) -> Result<(), MeterError> {
        let ctx = HidContext::new().map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let mut device = ctx
            .open_device(&I1_DISPLAY_PRO)
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

        // Set integration time if different
        if integration_time_ms != self.integration_time_ms {
            let payload = integration_time_ms.to_le_bytes();
            send_command(device, CMD_SET_INTEGRATION_TIME, &payload)
                .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
            let resp = read_response(device, 2000)
                .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
            if !resp.is_empty() && !XriteStatus::from_byte(resp[0]).is_ok() {
                return Err(MeterError::ConnectionFailed(
                    "Failed to set integration time".to_string(),
                ));
            }
            self.integration_time_ms = integration_time_ms;
        }

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

        // Parse XYZ from offsets 2, 6, 10 as float32
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
        "i1 Display Pro Rev.B"
    }
}
