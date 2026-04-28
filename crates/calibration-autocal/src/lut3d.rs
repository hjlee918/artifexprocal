use hal::types::{Lut3D, RGB};

/// Tetrahedral interpolator for 3D LUT lookup.
pub struct TetrahedralInterpolator {
    lut: Lut3D,
}

impl TetrahedralInterpolator {
    pub fn new(lut: Lut3D) -> Self {
        Self { lut }
    }

    /// Look up the corrected RGB value for a normalized input (r, g, b) in [0, 1].
    pub fn lookup(&self, r: f64, g: f64, b: f64) -> RGB {
        let size = self.lut.size;
        if size < 2 {
            return RGB { r: 0.0, g: 0.0, b: 0.0 };
        }

        let rf = r.clamp(0.0, 1.0) * (size - 1) as f64;
        let gf = g.clamp(0.0, 1.0) * (size - 1) as f64;
        let bf = b.clamp(0.0, 1.0) * (size - 1) as f64;

        let r0 = rf.floor() as usize;
        let g0 = gf.floor() as usize;
        let b0 = bf.floor() as usize;

        let r1 = (r0 + 1).min(size - 1);
        let g1 = (g0 + 1).min(size - 1);
        let b1 = (b0 + 1).min(size - 1);

        let dr = rf - r0 as f64;
        let dg = gf - g0 as f64;
        let db = bf - b0 as f64;

        let c000 = self.at(r0, g0, b0);
        let c001 = self.at(r0, g0, b1);
        let c010 = self.at(r0, g1, b0);
        let c011 = self.at(r0, g1, b1);
        let c100 = self.at(r1, g0, b0);
        let c101 = self.at(r1, g0, b1);
        let c110 = self.at(r1, g1, b0);
        let c111 = self.at(r1, g1, b1);

        let mut result = RGB { r: 0.0, g: 0.0, b: 0.0 };

        if dr >= dg && dg >= db {
            let w0 = 1.0 - dr;
            let w1 = dr - dg;
            let w2 = dg - db;
            let w3 = db;
            result.r = w0 * c000.r + w1 * c100.r + w2 * c110.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c100.g + w2 * c110.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c100.b + w2 * c110.b + w3 * c111.b;
        } else if dr >= db && db >= dg {
            let w0 = 1.0 - dr;
            let w1 = dr - db;
            let w2 = db - dg;
            let w3 = dg;
            result.r = w0 * c000.r + w1 * c100.r + w2 * c101.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c100.g + w2 * c101.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c100.b + w2 * c101.b + w3 * c111.b;
        } else if dg >= dr && dr >= db {
            let w0 = 1.0 - dg;
            let w1 = dg - dr;
            let w2 = dr - db;
            let w3 = db;
            result.r = w0 * c000.r + w1 * c010.r + w2 * c110.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c010.g + w2 * c110.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c010.b + w2 * c110.b + w3 * c111.b;
        } else if dg >= db && db >= dr {
            let w0 = 1.0 - dg;
            let w1 = dg - db;
            let w2 = db - dr;
            let w3 = dr;
            result.r = w0 * c000.r + w1 * c010.r + w2 * c011.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c010.g + w2 * c011.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c010.b + w2 * c011.b + w3 * c111.b;
        } else if db >= dr && dr >= dg {
            let w0 = 1.0 - db;
            let w1 = db - dr;
            let w2 = dr - dg;
            let w3 = dg;
            result.r = w0 * c000.r + w1 * c001.r + w2 * c101.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c001.g + w2 * c101.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c001.b + w2 * c101.b + w3 * c111.b;
        } else {
            // db >= dg && dg >= dr
            let w0 = 1.0 - db;
            let w1 = db - dg;
            let w2 = dg - dr;
            let w3 = dr;
            result.r = w0 * c000.r + w1 * c001.r + w2 * c011.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c001.g + w2 * c011.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c001.b + w2 * c011.b + w3 * c111.b;
        }

        result
    }

    /// Read a single LUT entry by grid coordinate, returning black if out of bounds.
    fn at(&self, r: usize, g: usize, b: usize) -> RGB {
        let idx = (r * self.lut.size + g) * self.lut.size + b;
        self.lut.data.get(idx).copied().unwrap_or(RGB { r: 0.0, g: 0.0, b: 0.0 })
    }
}
