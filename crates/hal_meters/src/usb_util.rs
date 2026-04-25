use rusb::{Context, DeviceHandle, UsbContext};

pub const GRETAGMACBETH_VID: u16 = 0x0971;
pub const I1_PRO_2_PID: u16 = 0x2000;

pub struct UsbDevice {
    handle: DeviceHandle<Context>,
}

#[derive(Debug, thiserror::Error)]
pub enum UsbUtilError {
    #[error("USB context init failed: {0}")]
    ContextInit(String),
    #[error("Device not found: VID {vid:04X}, PID {pid:04X}")]
    DeviceNotFound { vid: u16, pid: u16 },
    #[error("USB open failed: {0}")]
    OpenFailed(String),
    #[error("Claim interface failed: {0}")]
    ClaimFailed(String),
    #[error("Write failed: {0}")]
    WriteFailed(String),
    #[error("Read failed: {0}")]
    ReadFailed(String),
}

impl UsbDevice {
    pub fn open_xrite(vid: u16, pid: u16) -> Result<Self, UsbUtilError> {
        let context = Context::new().map_err(|e| UsbUtilError::ContextInit(e.to_string()))?;
        let handle = context
            .open_device_with_vid_pid(vid, pid)
            .ok_or(UsbUtilError::DeviceNotFound { vid, pid })?;
        handle
            .claim_interface(0)
            .map_err(|e| UsbUtilError::ClaimFailed(e.to_string()))?;
        Ok(Self { handle })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), UsbUtilError> {
        self.handle
            .write_bulk(0x01, data, std::time::Duration::from_millis(5000))
            .map_err(|e| UsbUtilError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, UsbUtilError> {
        let n = self
            .handle
            .read_bulk(0x82, buf, std::time::Duration::from_millis(10000))
            .map_err(|e| UsbUtilError::ReadFailed(e.to_string()))?;
        Ok(n)
    }
}

pub fn send_command_usb(device: &mut UsbDevice, cmd: u8, payload: &[u8]) -> Result<(), UsbUtilError> {
    let mut report = vec![0u8; 64];
    report[0] = cmd;
    let len = payload.len().min(63);
    report[1..1 + len].copy_from_slice(&payload[..len]);
    device.write(&report)?;
    Ok(())
}

pub fn read_response_usb(device: &mut UsbDevice, timeout_ms: i32) -> Result<Vec<u8>, UsbUtilError> {
    let mut buf = vec![0u8; 64];
    let start = std::time::Instant::now();
    loop {
        let n = device.read(&mut buf)?;
        if n > 0 {
            buf.truncate(n);
            return Ok(buf);
        }
        if start.elapsed().as_millis() > timeout_ms as u128 {
            return Err(UsbUtilError::ReadFailed("Timeout".to_string()));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
