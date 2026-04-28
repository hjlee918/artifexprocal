use calibration_core::patch::{Patch, PatchSet, PatchStrategy};
use color_science::types::RGB;

pub struct OptimizedPatchSetGenerator;

impl OptimizedPatchSetGenerator {
    /// Generate an optimized subset of patches for 3D LUT measurement.
    /// Includes grayscale ramp, primary axes, near-neutral emphasis, boundary samples.
    pub fn generate(strategy: PatchStrategy) -> PatchSet {
        match strategy {
            PatchStrategy::Grayscale(count) => Self::grayscale(count),
            PatchStrategy::OptimizedSubset { grayscale_count, color_count } => {
                Self::optimized_subset(grayscale_count, color_count)
            }
        }
    }

    fn grayscale(count: usize) -> PatchSet {
        let mut patches = Vec::with_capacity(count);
        for i in 0..count {
            let level = i as f64 / (count.saturating_sub(1).max(1) as f64);
            patches.push(Patch::new(RGB { r: level, g: level, b: level }));
        }
        PatchSet { patches }
    }

    fn optimized_subset(grayscale_count: usize, color_count: usize) -> PatchSet {
        let mut patches = Vec::with_capacity(grayscale_count + color_count);

        // 1. Grayscale ramp
        for i in 0..grayscale_count {
            let level = i as f64 / (grayscale_count.saturating_sub(1).max(1) as f64);
            patches.push(Patch::new(RGB { r: level, g: level, b: level }));
        }

        // 2. Primary axes (R, G, B ramps)
        for i in 1..=5 {
            let level = i as f64 / 6.0;
            patches.push(Patch::new(RGB { r: level, g: 0.0, b: 0.0 }));
            patches.push(Patch::new(RGB { r: 0.0, g: level, b: 0.0 }));
            patches.push(Patch::new(RGB { r: 0.0, g: 0.0, b: level }));
        }

        // 3. Near-neutral emphasis: dense sampling in central 20%
        let neutral_steps = (color_count / 4).max(8);
        for i in 0..neutral_steps {
            let t = i as f64 / (neutral_steps.saturating_sub(1).max(1) as f64);
            let center = 0.4 + t * 0.2; // 0.4 to 0.6 range
            let r = center + (t - 0.5) * 0.1;
            let g = center;
            let b = center - (t - 0.5) * 0.1;
            patches.push(Patch::new(RGB {
                r: r.clamp(0.0, 1.0),
                g: g.clamp(0.0, 1.0),
                b: b.clamp(0.0, 1.0),
            }));
        }

        // 4. Boundary samples: corners and edges
        let corners = vec![
            RGB { r: 0.0, g: 0.0, b: 0.0 },
            RGB { r: 1.0, g: 0.0, b: 0.0 },
            RGB { r: 0.0, g: 1.0, b: 0.0 },
            RGB { r: 0.0, g: 0.0, b: 1.0 },
            RGB { r: 1.0, g: 1.0, b: 0.0 },
            RGB { r: 1.0, g: 0.0, b: 1.0 },
            RGB { r: 0.0, g: 1.0, b: 1.0 },
            RGB { r: 1.0, g: 1.0, b: 1.0 },
        ];
        for c in corners {
            patches.push(Patch::new(c));
        }

        // 5. Random jitter samples to fill remaining quota
        let remaining = (grayscale_count + color_count).saturating_sub(patches.len());
        for i in 0..remaining {
            let t = (i + 1) as f64 / (remaining.saturating_sub(1).max(1) as f64 + 1.0);
            let r = (t * 7.0).sin() * 0.5 + 0.5;
            let g = (t * 11.0).sin() * 0.5 + 0.5;
            let b = (t * 13.0).sin() * 0.5 + 0.5;
            patches.push(Patch::new(RGB { r, g, b }));
        }

        PatchSet { patches }
    }
}
