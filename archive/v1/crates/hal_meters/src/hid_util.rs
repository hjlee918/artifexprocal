use hidapi::{HidApi, HidDevice};

#[derive(Debug, Clone, Copy)]
pub struct XriteDevice {
    pub vid: u16,
    pub pid: u16,
    pub name: &'static str,
}

pub const I1_DISPLAY_PRO: XriteDevice = XriteDevice {
    vid: 0x0765,
    pid: 0x5020,
    name: "i1 Display Pro Rev.B",
};

// Note: i1 Pro 2 is NOT a HID device; it uses USB bulk transfer.
// See usb_util.rs for i1 Pro 2 constants.

#[derive(Debug, thiserror::Error)]
pub enum HidUtilError {
    #[error("HID API init failed: {0}")]
    ApiInit(String),
    #[error("Device not found: {name} (VID {vid:04X}, PID {pid:04X})")]
    DeviceNotFound { name: String, vid: u16, pid: u16 },
    #[error("HID open failed: {0}")]
    OpenFailed(String),
    #[error("Write failed: {0}")]
    WriteFailed(String),
    #[error("Read failed: {0}")]
    ReadFailed(String),
}

pub struct HidContext {
    api: HidApi,
}

impl HidContext {
    pub fn new() -> Result<Self, HidUtilError> {
        let api = HidApi::new().map_err(|e| HidUtilError::ApiInit(e.to_string()))?;
        Ok(Self { api })
    }

    pub fn enumerate_xrite(&self) -> Vec<(hidapi::DeviceInfo, XriteDevice)> {
        let mut found = Vec::new();
        for info in self.api.device_list() {
            if info.vendor_id() == I1_DISPLAY_PRO.vid && info.product_id() == I1_DISPLAY_PRO.pid {
                found.push((info.clone(), I1_DISPLAY_PRO));
            }
        }
        found
    }

    pub fn open_device(&self, xrite: &XriteDevice) -> Result<SyncHidDevice, HidUtilError> {
        self.api
            .open(xrite.vid, xrite.pid)
            .map_err(|e| HidUtilError::OpenFailed(e.to_string()))
            .map(SyncHidDevice)
    }

    pub fn open_by_serial(&self, xrite: &XriteDevice, serial: &str) -> Result<SyncHidDevice, HidUtilError> {
        self.api
            .open_serial(xrite.vid, xrite.pid, serial)
            .map_err(|e| HidUtilError::OpenFailed(e.to_string()))
            .map(SyncHidDevice)
    }
}

pub struct SyncHidDevice(pub HidDevice);

unsafe impl Send for SyncHidDevice {}
unsafe impl Sync for SyncHidDevice {}

impl SyncHidDevice {
    pub fn inner_mut(&mut self) -> &mut HidDevice {
        &mut self.0
    }
}

pub fn send_command(device: &mut SyncHidDevice, cmd: u8, payload: &[u8]) -> Result<(), HidUtilError> {
    let mut report = vec![0u8; 64];
    report[0] = cmd;
    let len = payload.len().min(63);
    report[1..1 + len].copy_from_slice(&payload[..len]);
    device
        .inner_mut()
        .write(&report)
        .map_err(|e| HidUtilError::WriteFailed(e.to_string()))?;
    Ok(())
}

/// Send a 16-bit command code (high byte in report[0], low byte in report[1]).
/// Payload starts at report[2]. Used for i1d3 unlock protocol.
pub fn send_command_u16(device: &mut SyncHidDevice, cmd: u16, payload: &[u8]) -> Result<(), HidUtilError> {
    let mut report = vec![0u8; 64];
    report[0] = ((cmd >> 8) & 0xFF) as u8;
    report[1] = (cmd & 0xFF) as u8;
    let len = payload.len().min(62);
    report[2..2 + len].copy_from_slice(&payload[..len]);
    device
        .inner_mut()
        .write(&report)
        .map_err(|e| HidUtilError::WriteFailed(e.to_string()))?;
    Ok(())
}

pub fn read_response(device: &mut SyncHidDevice, timeout_ms: i32) -> Result<Vec<u8>, HidUtilError> {
    let mut buf = vec![0u8; 64];
    let n = device
        .inner_mut()
        .read_timeout(&mut buf, timeout_ms)
        .map_err(|e| HidUtilError::ReadFailed(e.to_string()))?;
    buf.truncate(n);
    Ok(buf)
}
