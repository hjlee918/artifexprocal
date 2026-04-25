#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XYZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(non_snake_case)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XyY {
    pub x: f64,
    pub y: f64,
    pub Y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(non_snake_case)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lab {
    pub L: f64,
    pub a: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RGB {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WhitePoint {
    D65,
    D50,
    D93,
    Custom { x: f64, y: f64 },
}

impl WhitePoint {
    pub fn to_xyz(&self) -> XYZ {
        match self {
            WhitePoint::D65 => XYZ { x: 95.047, y: 100.0, z: 108.883 },
            WhitePoint::D50 => XYZ { x: 96.4212, y: 100.0, z: 82.5188 },
            WhitePoint::D93 => XYZ { x: 109.850, y: 100.0, z: 35.585 },
            WhitePoint::Custom { x, y } => {
                #[allow(non_snake_case)]
                let Y = 100.0;
                let z = 1.0 - x - y;
                XYZ {
                    x: (x / y) * Y,
                    y: Y,
                    z: (z / y) * Y,
                }
            }
        }
    }

    pub fn to_xy(&self) -> (f64, f64) {
        match self {
            WhitePoint::D65 => (0.3127, 0.3290),
            WhitePoint::D50 => (0.3457, 0.3585),
            WhitePoint::D93 => (0.2831, 0.2971),
            WhitePoint::Custom { x, y } => (*x, *y),
        }
    }
}

/// Standard illuminants for Lab conversions
pub fn illuminant_d65() -> XYZ {
    WhitePoint::D65.to_xyz()
}
