use crate::types::{XYZ, XyY};

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
