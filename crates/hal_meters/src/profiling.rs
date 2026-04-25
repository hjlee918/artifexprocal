use color_science::types::XYZ;

pub struct CorrectionMatrix {
    pub m: [[f64; 3]; 3],
}

impl CorrectionMatrix {
    pub fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn apply(&self, xyz: &XYZ) -> XYZ {
        XYZ {
            x: self.m[0][0] * xyz.x + self.m[0][1] * xyz.y + self.m[0][2] * xyz.z,
            y: self.m[1][0] * xyz.x + self.m[1][1] * xyz.y + self.m[1][2] * xyz.z,
            z: self.m[2][0] * xyz.x + self.m[2][1] * xyz.y + self.m[2][2] * xyz.z,
        }
    }
}

/// Stub: will be fully implemented in profiling phase
pub fn generate_correction_matrix(
    _spectro_xyz: &[XYZ],
    _colorimeter_xyz: &[XYZ],
) -> CorrectionMatrix {
    CorrectionMatrix::identity()
}
