use crate::types::{XYZ, XyY, Lab, WhitePoint};

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
