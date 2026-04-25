pub const CMD_GET_FIRMWARE: u8 = 0x01;
pub const CMD_SET_EMISSIVE: u8 = 0x02;
pub const CMD_TRIGGER_MEASURE: u8 = 0x03;
pub const CMD_READ_XYZ: u8 = 0x04;
pub const CMD_READ_SPECTRUM: u8 = 0x05;
pub const CMD_INITIALIZE: u8 = 0x06;
pub const CMD_SET_INTEGRATION_TIME: u8 = 0x07;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum XriteStatus {
    Ok = 0x00,
    Busy = 0x01,
    Error = 0xFF,
    InitializationRequired = 0xFE,
}

impl XriteStatus {
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x00 => Self::Ok,
            0x01 => Self::Busy,
            0xFE => Self::InitializationRequired,
            _ => Self::Error,
        }
    }

    pub fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}
