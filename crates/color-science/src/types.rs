//! Core colorimetric types and constants.

use serde::{Deserialize, Serialize};

/// CIE XYZ tristimulus values.
/// Y is luminance in cd/m² for emissive sources.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Xyz {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// CIE xyY — chromaticity + luminance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct XyY {
    pub x: f64,
    pub y: f64,
    pub y_lum: f64,
}

/// CIELAB (D65 reference white).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// CIE LCh (polar form of CIELAB).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LCh {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

/// CIE 1976 u′v′ (UCS) chromaticity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UvPrime {
    pub u: f64,
    pub v: f64,
}

/// ICtCp perceptual color difference space (for HDR).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ICtCp {
    pub i: f64,
    pub ct: f64,
    pub cp: f64,
}

/// RGB triplet with generic channel type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

/// Standard RGB color spaces.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RgbSpace {
    Srgb,
    Rec709,
    Rec2020,
    DciP3,
    DisplayP3,
    AdobeRgb,
    ProPhoto,
}

/// Reference white points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WhitePoint {
    D50,
    D55,
    D65,
    D75,
    C,
    E,
}

impl WhitePoint {
    /// CIE 1931 2° observer tristimulus values.
    pub fn xyz(&self) -> Xyz {
        match self {
            WhitePoint::D50 => Xyz { x: 96.4212, y: 100.0, z: 82.5188 },
            WhitePoint::D55 => Xyz { x: 95.6797, y: 100.0, z: 92.1481 },
            WhitePoint::D65 => Xyz { x: 95.047,  y: 100.0, z: 108.883 },
            WhitePoint::D75 => Xyz { x: 94.972,  y: 100.0, z: 122.639 },
            WhitePoint::C   => Xyz { x: 98.074,  y: 100.0, z: 118.232 },
            WhitePoint::E   => Xyz { x: 100.0,   y: 100.0, z: 100.0 },
        }
    }
}

/// D65 reference white (CIE 2° standard observer).
pub const D65: Xyz = Xyz {
    x: 95.047,
    y: 100.0,
    z: 108.883,
};
