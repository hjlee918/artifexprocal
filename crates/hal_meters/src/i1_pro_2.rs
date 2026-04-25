use hal::traits::Meter;
use hal::error::MeterError;
use color_science::types::XYZ;
use crate::spectro_trait::Spectrophotometer;

#[cfg(not(target_os = "macos"))]
use crate::usb_util::{UsbDevice, GRETAGMACBETH_VID, I1_PRO_2_PID, send_command_usb, read_response_usb};
#[cfg(not(target_os = "macos"))]
use crate::commands::{CMD_GET_FIRMWARE, CMD_SET_EMISSIVE, CMD_TRIGGER_MEASURE, CMD_INITIALIZE, XriteStatus};

#[cfg(target_os = "macos")]
use crate::argyll_adapter::{ArgyllMeter, ArgyllPort};

#[cfg(not(target_os = "macos"))]
pub struct SyncUsbDevice(UsbDevice);
#[cfg(not(target_os = "macos"))]
unsafe impl Send for SyncUsbDevice {}
#[cfg(not(target_os = "macos"))]
unsafe impl Sync for SyncUsbDevice {}

#[cfg(not(target_os = "macos"))]
impl SyncUsbDevice {
    pub fn inner_mut(&mut self) -> &mut UsbDevice {
        &mut self.0
    }
}

#[cfg(not(target_os = "macos"))]
pub struct I1Pro2 {
    device: Option<SyncUsbDevice>,
    serial: Option<String>,
    connected: bool,
}

#[cfg(target_os = "macos")]
pub struct I1Pro2 {
    adapter: ArgyllMeter,
}

impl I1Pro2 {
    pub fn new() -> Self {
        #[cfg(not(target_os = "macos"))]
        {
            Self {
                device: None,
                serial: None,
                connected: false,
            }
        }
        #[cfg(target_os = "macos")]
        {
            Self {
                adapter: ArgyllMeter::new(ArgyllPort::i1_pro_2(), "i1 Pro 2"),
            }
        }
    }

    pub fn serial(&self) -> Option<&str> {
        #[cfg(not(target_os = "macos"))]
        {
            self.serial.as_deref()
        }
        #[cfg(target_os = "macos")]
        {
            None
        }
    }

    pub fn initialize(&mut self) -> Result<(), MeterError> {
        #[cfg(not(target_os = "macos"))]
        {
            let device = self.device.as_mut().ok_or_else(|| {
                MeterError::ConnectionFailed("Device not open".to_string())
            })?;

            send_command_usb(device.inner_mut(), CMD_INITIALIZE, &[])
                .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
            let resp = read_response_usb(device.inner_mut(), 10000)
                .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
            if resp.is_empty() || !XriteStatus::from_byte(resp[0]).is_ok() {
                return Err(MeterError::ConnectionFailed(
                    "Initialization failed".to_string(),
                ));
            }

            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            self.adapter.initialize()
                .map_err(|e| MeterError::ConnectionFailed(e.to_string()))
        }
    }

    pub fn read_spectrum_raw(&mut self) -> Result<[f64; 36], MeterError> {
        #[cfg(not(target_os = "macos"))]
        {
            let device = self.device.as_mut().ok_or_else(|| {
                MeterError::ConnectionFailed("Device not open".to_string())
            })?;

            send_command_usb(device.inner_mut(), CMD_TRIGGER_MEASURE, &[])
                .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
            let resp = read_response_usb(device.inner_mut(), 8000)
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
        #[cfg(target_os = "macos")]
        {
            self.adapter.read_spectrum()
                .map_err(|e| MeterError::InvalidResponse(e.to_string()))
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl Meter for I1Pro2 {
    fn connect(&mut self) -> Result<(), MeterError> {
        let device = UsbDevice::open_xrite(GRETAGMACBETH_VID, I1_PRO_2_PID)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;

        let mut wrapped = SyncUsbDevice(device);

        // Verify firmware
        send_command_usb(wrapped.inner_mut(), CMD_GET_FIRMWARE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response_usb(wrapped.inner_mut(), 2000)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        if resp.is_empty() || XriteStatus::from_byte(resp[0]).is_ok() {
            // Firmware response received
        }

        // Set emissive mode
        send_command_usb(wrapped.inner_mut(), CMD_SET_EMISSIVE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response_usb(wrapped.inner_mut(), 2000)
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        if !resp.is_empty() && !XriteStatus::from_byte(resp[0]).is_ok() {
            return Err(MeterError::ConnectionFailed(
                "Failed to set emissive mode".to_string(),
            ));
        }

        self.device = Some(wrapped);
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.device = None;
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
        send_command_usb(device.inner_mut(), CMD_TRIGGER_MEASURE, &[])
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))?;
        let resp = read_response_usb(device.inner_mut(), integration_time_ms as i32 + 2000)
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

#[cfg(target_os = "macos")]
impl Meter for I1Pro2 {
    fn connect(&mut self) -> Result<(), MeterError> {
        self.adapter.connect()
            .map_err(|e| MeterError::ConnectionFailed(e.to_string()))
    }

    fn disconnect(&mut self) {
        self.adapter.disconnect();
    }

    fn read_xyz(&mut self, _integration_time_ms: u32) -> Result<XYZ, MeterError> {
        self.adapter.read_xyz(_integration_time_ms)
            .map_err(|e| MeterError::InvalidResponse(e.to_string()))
    }

    fn model(&self) -> &str {
        self.adapter.model()
    }
}

#[cfg(not(target_os = "macos"))]
impl Spectrophotometer for I1Pro2 {
    fn read_spectrum(&mut self) -> Result<[f64; 36], MeterError> {
        self.read_spectrum_raw()
    }
}

#[cfg(target_os = "macos")]
impl Spectrophotometer for I1Pro2 {
    fn read_spectrum(&mut self) -> Result<[f64; 36], MeterError> {
        self.adapter.read_spectrum()
            .map_err(|e| MeterError::InvalidResponse(e.to_string()))
    }
}
