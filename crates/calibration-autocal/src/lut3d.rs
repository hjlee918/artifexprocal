use color_science::types::{XYZ, RGB};
use hal::types::Lut3D;

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

/// Pure engine for computing a dense 3D LUT from sparse measured patches.
pub struct Lut3DEngine;

impl Lut3DEngine {
    /// Compute a Lut3D of the given size from measured patches.
    /// Patches should be in normalized RGB [0,1] with corresponding XYZ measurements.
    pub fn compute(
        patches: &[(RGB, XYZ)],
        size: usize,
        _target_space: &calibration_core::state::TargetSpace,
    ) -> Result<Lut3D, String> {
        if size < 2 {
            return Err("LUT size must be at least 2".to_string());
        }
        if patches.is_empty() {
            return Err("No patches provided".to_string());
        }

        // Build sparse mapping: target RGB -> measured XYZ
        let sparse: Vec<(RGB, XYZ)> = patches.to_vec();

        // For each grid point, interpolate in XYZ space using nearest neighbors
        let mut data = Vec::with_capacity(size * size * size);

        for r in 0..size {
            for g in 0..size {
                for b in 0..size {
                    let rf = r as f64 / (size - 1) as f64;
                    let gf = g as f64 / (size - 1) as f64;
                    let bf = b as f64 / (size - 1) as f64;

                    let target_rgb = RGB { r: rf, g: gf, b: bf };

                    // Find nearest measured neighbors and interpolate
                    let measured_xyz = Self::interpolate_xyz(&sparse, rf, gf, bf);

                    // Compute correction: target RGB -> what RGB produces measured XYZ
                    // For MVP: simple ratio correction (identity for perfect display)
                    let corrected = Self::compute_correction(target_rgb, measured_xyz);

                    data.push(corrected);
                }
            }
        }

        Ok(Lut3D { data, size })
    }

    fn interpolate_xyz(sparse: &[(RGB, XYZ)], r: f64, g: f64, b: f64) -> XYZ {
        // Inverse distance weighting with k=4 nearest neighbors
        let mut neighbors: Vec<(f64, XYZ)> = sparse.iter()
            .map(|(rgb, xyz)| {
                let dist_sq = (rgb.r - r).powi(2) + (rgb.g - g).powi(2) + (rgb.b - b).powi(2);
                let dist = dist_sq.sqrt().max(1e-10);
                (dist, *xyz)
            })
            .collect();

        neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        neighbors.truncate(4);

        let total_weight: f64 = neighbors.iter().map(|(d, _)| 1.0 / d).sum();
        if total_weight == 0.0 {
            return XYZ { x: 0.0, y: 0.0, z: 0.0 };
        }

        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        for (dist, xyz) in &neighbors {
            let w = 1.0 / dist;
            x += xyz.x * w;
            y += xyz.y * w;
            z += xyz.z * w;
        }

        XYZ {
            x: x / total_weight,
            y: y / total_weight,
            z: z / total_weight,
        }
    }

    fn compute_correction(target_rgb: RGB, measured_xyz: XYZ) -> RGB {
        // MVP: compute simple correction assuming display response is approximately linear
        // For a perfect display, measured XYZ should correspond to target RGB
        // Correction = target / measured (normalized)
        // This is a simplified approach; real implementation would invert the full
        // display transform using XYZ->RGB matrix.
        let sum = measured_xyz.x + measured_xyz.y + measured_xyz.z;
        if sum < 1e-10 {
            return target_rgb;
        }

        // Normalized measured "RGB" (very rough approximation for correction magnitude)
        let mr = measured_xyz.x / sum;
        let mg = measured_xyz.y / sum;
        let mb = measured_xyz.z / sum;

        RGB {
            r: (target_rgb.r / mr.max(0.01)).clamp(0.0, 1.0),
            g: (target_rgb.g / mg.max(0.01)).clamp(0.0, 1.0),
            b: (target_rgb.b / mb.max(0.01)).clamp(0.0, 1.0),
        }
    }
}
