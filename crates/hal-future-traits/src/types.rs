//! Shared types for future HAL drivers (design sketches).

use serde::{Deserialize, Serialize};

/// A 1D LUT (per-channel tone curve).
#[derive(Debug, Clone, PartialEq)]
pub struct Lut1D {
    pub size: usize,
    pub channels: [Vec<f64>; 3], // R, G, B
}

/// A 3D LUT (volumetric color correction cube).
#[derive(Debug, Clone, PartialEq)]
pub struct Lut3D {
    pub size: usize,
    pub data: Vec<f64>, // size³ × 3 (R, G, B)
}

/// Picture modes supported by display controllers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PictureMode {
    Expert1,
    Expert2,
    Cinema,
    Game,
    HdrStandard,
    HdrCinema,
    DolbyVision,
}

impl PictureMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PictureMode::Expert1 => "expert1",
            PictureMode::Expert2 => "expert2",
            PictureMode::Cinema => "cinema",
            PictureMode::Game => "game",
            PictureMode::HdrStandard => "hdrStandard",
            PictureMode::HdrCinema => "hdrCinema",
            PictureMode::DolbyVision => "dolbyVision",
        }
    }
}

/// Display calibration data bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationData {
    pub picture_mode: PictureMode,
    pub lut_1d: Option<Lut1D>,
    pub lut_3d_bt709: Option<Lut3D>,
    pub lut_3d_bt2020: Option<Lut3D>,
    pub gamut_matrix: Option<[[f64; 3]; 3]>,
    pub white_balance: Option<WhiteBalance>,
}

/// White balance settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhiteBalance {
    pub gain_r: f64,
    pub gain_g: f64,
    pub gain_b: f64,
    pub offset_r: f64,
    pub offset_g: f64,
    pub offset_b: f64,
}

/// A test patch for pattern generators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Patch {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub bit_depth: u8,
}

impl Patch {
    /// Create an 8-bit patch.
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as u16,
            g: g as u16,
            b: b as u16,
            bit_depth: 8,
        }
    }

    /// Create a 10-bit patch.
    pub fn rgb10(r: u16, g: u16, b: u16) -> Self {
        Self {
            r, g, b,
            bit_depth: 10,
        }
    }

    /// Normalize to 0.0–1.0 range.
    pub fn normalized(&self) -> (f64, f64, f64) {
        let max = ((1u32 << self.bit_depth) - 1) as f64;
        (
            self.r as f64 / max,
            self.g as f64 / max,
            self.b as f64 / max,
        )
    }
}
