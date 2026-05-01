//! Colorspace conversions: XYZ ↔ xyY ↔ Lab ↔ LCh ↔ u′v′.

use crate::types::{ICtCp, Lab, LCh, UvPrime, WhitePoint, XyY, Xyz};

// ── XYZ ↔ xyY ────────────────────────────────────────────────

impl From<Xyz> for XyY {
    fn from(xyz: Xyz) -> Self {
        let sum = xyz.x + xyz.y + xyz.z;
        if sum == 0.0 {
            XyY { x: 0.0, y: 0.0, y_lum: xyz.y }
        } else {
            XyY {
                x: xyz.x / sum,
                y: xyz.y / sum,
                y_lum: xyz.y,
            }
        }
    }
}

impl From<XyY> for Xyz {
    fn from(xyy: XyY) -> Self {
        if xyy.y == 0.0 {
            Xyz { x: 0.0, y: 0.0, z: 0.0 }
        } else {
            let factor = xyy.y_lum / xyy.y;
            Xyz {
                x: xyy.x * factor,
                y: xyy.y_lum,
                z: (1.0 - xyy.x - xyy.y) * factor,
            }
        }
    }
}

// ── XYZ ↔ Lab (CIELAB, D65 default) ──────────────────────────

/// Convert XYZ to Lab with a given reference white.
pub fn xyz_to_lab(xyz: Xyz, white: Xyz) -> Lab {
    let fx = f_lab(xyz.x / white.x);
    let fy = f_lab(xyz.y / white.y);
    let fz = f_lab(xyz.z / white.z);
    Lab {
        l: 116.0 * fy - 16.0,
        a: 500.0 * (fx - fy),
        b: 200.0 * (fy - fz),
    }
}

/// Convert Lab to XYZ with a given reference white.
pub fn lab_to_xyz(lab: Lab, white: Xyz) -> Xyz {
    let fy = (lab.l + 16.0) / 116.0;
    let fx = lab.a / 500.0 + fy;
    let fz = fy - lab.b / 200.0;
    Xyz {
        x: white.x * f_lab_inv(fx),
        y: white.y * f_lab_inv(fy),
        z: white.z * f_lab_inv(fz),
    }
}

fn f_lab(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta.powi(3) {
        t.cbrt()
    } else {
        t / (3.0 * delta.powi(2)) + 4.0 / 29.0
    }
}

fn f_lab_inv(t: f64) -> f64 {
    let delta = 6.0 / 29.0;
    if t > delta {
        t.powi(3)
    } else {
        3.0 * delta.powi(2) * (t - 4.0 / 29.0)
    }
}

// ── Lab ↔ LCh ────────────────────────────────────────────────

impl From<Lab> for LCh {
    fn from(lab: Lab) -> Self {
        let c = (lab.a.powi(2) + lab.b.powi(2)).sqrt();
        let h = lab.b.atan2(lab.a).to_degrees();
        let h = if h < 0.0 { h + 360.0 } else { h };
        LCh { l: lab.l, c, h }
    }
}

impl From<LCh> for Lab {
    fn from(lch: LCh) -> Self {
        let rad = lch.h.to_radians();
        Lab {
            l: lch.l,
            a: lch.c * rad.cos(),
            b: lch.c * rad.sin(),
        }
    }
}

// ── XYZ ↔ u′v′ (CIE 1976 UCS) ───────────────────────────────

pub fn xyz_to_uv_prime(xyz: Xyz) -> UvPrime {
    let denom = xyz.x + 15.0 * xyz.y + 3.0 * xyz.z;
    if denom == 0.0 {
        UvPrime { u: 0.0, v: 0.0 }
    } else {
        UvPrime {
            u: 4.0 * xyz.x / denom,
            v: 9.0 * xyz.y / denom,
        }
    }
}

pub fn uv_prime_to_xyz(uv: UvPrime, y: f64) -> Xyz {
    if uv.v == 0.0 {
        Xyz { x: 0.0, y, z: 0.0 }
    } else {
        let x = 9.0 * uv.u * y / (4.0 * uv.v);
        let z = (12.0 - 3.0 * uv.u - 20.0 * uv.v) * y / (4.0 * uv.v);
        Xyz { x, y, z }
    }
}

// ── Convenience: D65 default Lab ─────────────────────────────

impl From<Xyz> for Lab {
    fn from(xyz: Xyz) -> Self {
        xyz_to_lab(xyz, WhitePoint::D65.xyz())
    }
}

impl From<Lab> for Xyz {
    fn from(lab: Lab) -> Self {
        lab_to_xyz(lab, WhitePoint::D65.xyz())
    }
}

// ── ICtCp (Phase 1: stub, full impl in Phase 2) ──────────────

/// Convert XYZ to ICtCp.
/// This is a simplified version for Phase 1. Full HDR-aware conversion
/// with BT.2020/PQ will be implemented in Phase 2.
pub fn xyz_to_ictcp(xyz: Xyz) -> ICtCp {
    // LMS conversion (simplified, not PQ-encoded)
    let l = 0.8189330101 * xyz.x + 0.3618667424 * xyz.y - 0.1288597137 * xyz.z;
    let m = 0.0329845436 * xyz.x + 0.9293118715 * xyz.y + 0.0361456387 * xyz.z;
    let s = 0.0482003018 * xyz.x + 0.2643662691 * xyz.y + 0.6338517070 * xyz.z;

    // ICtCp matrix (simplified, non-PQ)
    let i = 0.5 * l + 0.5 * m;
    let ct = 0.5 * l - 0.5 * m;
    let cp = 0.25 * l + 0.25 * m - 0.5 * s;

    ICtCp { i, ct, cp }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Reference values: D65 ──────────────────────────────────
    const D65_XYZ: Xyz = Xyz { x: 95.047, y: 100.0, z: 108.883 };

    #[test]
    fn xyz_xyy_roundtrip() {
        let xyy: XyY = D65_XYZ.into();
        let xyz: Xyz = xyy.into();
        assert!((xyz.x - D65_XYZ.x).abs() < 1e-10);
        assert!((xyz.y - D65_XYZ.y).abs() < 1e-10);
        assert!((xyz.z - D65_XYZ.z).abs() < 1e-10);
    }

    #[test]
    fn d65_xyy_is_correct() {
        let xyy: XyY = D65_XYZ.into();
        assert!((xyy.x - 0.3127).abs() < 1e-4);
        assert!((xyy.y - 0.3290).abs() < 1e-4);
        assert!((xyy.y_lum - 100.0).abs() < 1e-10);
    }

    #[test]
    fn xyz_lab_roundtrip() {
        let lab: Lab = D65_XYZ.into();
        let xyz: Xyz = lab.into();
        assert!((xyz.x - D65_XYZ.x).abs() < 1e-6);
        assert!((xyz.y - D65_XYZ.y).abs() < 1e-6);
        assert!((xyz.z - D65_XYZ.z).abs() < 1e-6);
    }

    #[test]
    fn d65_lab_is_white() {
        let lab: Lab = D65_XYZ.into();
        assert!((lab.l - 100.0).abs() < 1e-6);
        assert!(lab.a.abs() < 1e-6);
        assert!(lab.b.abs() < 1e-6);
    }

    #[test]
    fn lab_lch_roundtrip() {
        let lab = Lab { l: 50.0, a: 20.0, b: -30.0 };
        let lch: LCh = lab.into();
        let back: Lab = lch.into();
        assert!((back.l - lab.l).abs() < 1e-10);
        assert!((back.a - lab.a).abs() < 1e-10);
        assert!((back.b - lab.b).abs() < 1e-10);
    }

    #[test]
    fn xyz_uv_roundtrip() {
        let uv = xyz_to_uv_prime(D65_XYZ);
        let xyz = uv_prime_to_xyz(uv, D65_XYZ.y);
        assert!((xyz.x - D65_XYZ.x).abs() < 1e-6);
        assert!((xyz.y - D65_XYZ.y).abs() < 1e-6);
        assert!((xyz.z - D65_XYZ.z).abs() < 1e-6);
    }

    #[test]
    fn d65_uv_prime_is_correct() {
        let uv = xyz_to_uv_prime(D65_XYZ);
        assert!((uv.u - 0.1978).abs() < 1e-4);
        assert!((uv.v - 0.4683).abs() < 1e-4);
    }
}
