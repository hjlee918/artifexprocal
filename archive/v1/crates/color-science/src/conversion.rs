use crate::types::{XYZ, XyY, Lab, RGB, WhitePoint};
use crate::gamma::{srgb_gamma_encode, srgb_gamma_decode};

impl XYZ {
    /// Convert XYZ to xyY chromaticity coordinates
    pub fn to_xyy(&self) -> XyY {
        let sum = self.x + self.y + self.z;
        if sum == 0.0 {
            XyY { x: 0.0, y: 0.0, Y: 0.0 }
        } else {
            XyY {
                x: self.x / sum,
                y: self.y / sum,
                Y: self.y,
            }
        }
    }
}

impl XyY {
    /// Convert xyY to XYZ
    pub fn to_xyz(&self) -> XYZ {
        if self.y == 0.0 {
            XYZ { x: 0.0, y: 0.0, z: 0.0 }
        } else {
            XYZ {
                x: (self.x / self.y) * self.Y,
                y: self.Y,
                z: ((1.0 - self.x - self.y) / self.y) * self.Y,
            }
        }
    }
}

fn lab_f(t: f64) -> f64 {
    const DELTA: f64 = 6.0 / 29.0;
    const DELTA_SQ: f64 = DELTA * DELTA;
    const DELTA_CB: f64 = DELTA * DELTA * DELTA;

    if t > DELTA_CB {
        t.cbrt()
    } else {
        t / (3.0 * DELTA_SQ) + 4.0 / 29.0
    }
}

fn lab_f_inv(t: f64) -> f64 {
    const DELTA: f64 = 6.0 / 29.0;
    const DELTA_SQ: f64 = DELTA * DELTA;

    if t > DELTA {
        t * t * t
    } else {
        3.0 * DELTA_SQ * (t - 4.0 / 29.0)
    }
}

impl XYZ {
    pub fn to_lab(&self, white: WhitePoint) -> Lab {
        let xyz_n = white.to_xyz();

        let fx = lab_f(self.x / xyz_n.x);
        let fy = lab_f(self.y / xyz_n.y);
        let fz = lab_f(self.z / xyz_n.z);

        Lab {
            L: 116.0 * fy - 16.0,
            a: 500.0 * (fx - fy),
            b: 200.0 * (fy - fz),
        }
    }
}

impl Lab {
    pub fn to_xyz(&self, white: WhitePoint) -> XYZ {
        let xyz_n = white.to_xyz();

        let fy = (self.L + 16.0) / 116.0;
        let fx = self.a / 500.0 + fy;
        let fz = fy - self.b / 200.0;

        XYZ {
            x: xyz_n.x * lab_f_inv(fx),
            y: xyz_n.y * lab_f_inv(fy),
            z: xyz_n.z * lab_f_inv(fz),
        }
    }
}

/// sRGB D65 primaries to XYZ conversion matrix (scaled by 100 so white Y=100)
const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [41.24564, 35.75761, 18.04375],
    [21.26729, 71.51522, 7.21750],
    [1.93339, 11.91920, 95.03041],
];

const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [0.032404542, -0.015371385, -0.004985314],
    [-0.009692660, 0.018760108, 0.000415560],
    [0.000556434, -0.002040259, 0.010572252],
];

impl RGB {
    /// Convert linear RGB to XYZ (D65, sRGB primaries)
    pub fn to_xyz_srgb(&self) -> XYZ {
        XYZ {
            x: SRGB_TO_XYZ[0][0] * self.r + SRGB_TO_XYZ[0][1] * self.g + SRGB_TO_XYZ[0][2] * self.b,
            y: SRGB_TO_XYZ[1][0] * self.r + SRGB_TO_XYZ[1][1] * self.g + SRGB_TO_XYZ[1][2] * self.b,
            z: SRGB_TO_XYZ[2][0] * self.r + SRGB_TO_XYZ[2][1] * self.g + SRGB_TO_XYZ[2][2] * self.b,
        }
    }

    /// Convert gamma-encoded sRGB to XYZ (applies inverse gamma first)
    pub fn to_xyz_from_encoded_srgb(&self) -> XYZ {
        let linear = RGB {
            r: srgb_gamma_decode(self.r),
            g: srgb_gamma_decode(self.g),
            b: srgb_gamma_decode(self.b),
        };
        linear.to_xyz_srgb()
    }
}

impl XYZ {
    /// Convert XYZ to linear sRGB RGB
    pub fn to_rgb_srgb(&self) -> RGB {
        RGB {
            r: XYZ_TO_SRGB[0][0] * self.x + XYZ_TO_SRGB[0][1] * self.y + XYZ_TO_SRGB[0][2] * self.z,
            g: XYZ_TO_SRGB[1][0] * self.x + XYZ_TO_SRGB[1][1] * self.y + XYZ_TO_SRGB[1][2] * self.z,
            b: XYZ_TO_SRGB[2][0] * self.x + XYZ_TO_SRGB[2][1] * self.y + XYZ_TO_SRGB[2][2] * self.z,
        }
    }

    /// Convert XYZ to gamma-encoded sRGB (applies gamma encoding)
    pub fn to_encoded_rgb_srgb(&self) -> RGB {
        let linear = self.to_rgb_srgb();
        RGB {
            r: srgb_gamma_encode(linear.r),
            g: srgb_gamma_encode(linear.g),
            b: srgb_gamma_encode(linear.b),
        }
    }
}
