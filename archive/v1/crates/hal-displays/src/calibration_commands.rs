use hal::types::{Lut1D, Lut3D, RGBGain};

#[derive(Debug, Clone, Default)]
pub struct CalibrationMode {
    active: bool,
    pic_mode: Option<String>,
}

impl CalibrationMode {
    pub fn new() -> Self {
        Self { active: false, pic_mode: None }
    }

    pub fn start(&mut self, pic_mode: &str) {
        self.active = true;
        self.pic_mode = Some(pic_mode.to_string());
    }

    pub fn end(&mut self) {
        self.active = false;
        self.pic_mode = None;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_inactive(&self) -> bool {
        !self.active
    }

    pub fn pic_mode(&self) -> Option<&str> {
        self.pic_mode.as_deref()
    }
}

pub fn encode_1d_lut(lut: &Lut1D) -> Vec<u8> {
    let mut data = Vec::with_capacity(lut.size * 3 * 8);
    for ch in 0..3 {
        for &val in &lut.channels[ch] {
            data.extend_from_slice(&val.to_le_bytes());
        }
    }
    data
}

pub fn encode_3d_lut(lut: &Lut3D) -> Vec<u8> {
    let mut data = Vec::with_capacity(lut.data.len() * 3 * 8);
    for rgb in &lut.data {
        data.extend_from_slice(&rgb.r.to_le_bytes());
        data.extend_from_slice(&rgb.g.to_le_bytes());
        data.extend_from_slice(&rgb.b.to_le_bytes());
    }
    data
}

pub fn encode_white_balance(gains: &RGBGain) -> (u16, u16, u16) {
    let r = ((gains.r / 2.0).clamp(0.0, 1.0) * 65535.0).round() as u16;
    let g = ((gains.g / 2.0).clamp(0.0, 1.0) * 65535.0).round() as u16;
    let b = ((gains.b / 2.0).clamp(0.0, 1.0) * 65535.0).round() as u16;
    (r, g, b)
}
