use calibration_core::state::{TargetSpace, WhitePoint};
use color_science::types::{XYZ, RGB};
use color_science::types::WhitePoint as CsWhitePoint;
use color_science::delta_e;

#[derive(Debug, Clone)]
pub struct GreyscaleAnalysis {
    pub gamma: f64,
    pub max_de: f64,
    pub avg_de: f64,
    pub white_balance_errors: Vec<f64>,
    pub per_channel_corrections: [Vec<f64>; 3],
}

pub struct GreyscaleAnalyzer;

fn to_cs_white_point(wp: &WhitePoint) -> CsWhitePoint {
    match wp {
        WhitePoint::D65 => CsWhitePoint::D65,
        WhitePoint::D50 => CsWhitePoint::D50,
        WhitePoint::Dci => CsWhitePoint::Custom { x: 0.3140, y: 0.3510 },
        WhitePoint::Custom(xyz) => CsWhitePoint::Custom {
            x: xyz.x / (xyz.x + xyz.y + xyz.z),
            y: xyz.y / (xyz.x + xyz.y + xyz.z),
        },
    }
}

impl GreyscaleAnalyzer {
    pub fn analyze(
        readings: &[(RGB, XYZ)],
        _target: &TargetSpace,
        white_point: &WhitePoint,
    ) -> Result<GreyscaleAnalysis, String> {
        if readings.is_empty() {
            return Err("No readings provided".to_string());
        }

        let cs_wp = to_cs_white_point(white_point);
        let white_xyz = readings.last().unwrap().1;
        let lab_ref = white_xyz.to_lab(cs_wp);

        let mut max_de: f64 = 0.0;
        let mut total_de: f64 = 0.0;
        let mut errors = Vec::with_capacity(readings.len());

        for (_rgb, xyz) in readings {
            let lab = xyz.to_lab(cs_wp);
            let de = delta_e::delta_e_2000(&lab_ref, &lab);
            max_de = max_de.max(de);
            total_de += de;
            errors.push(de);
        }

        let avg_de = total_de / readings.len() as f64;

        // Estimate gamma from log-log fit of Y vs input level
        let mut gamma_estimate = 2.2;
        let mut valid_pairs = Vec::new();
        for (rgb, xyz) in readings {
            if rgb.r > 0.0 && xyz.y > 0.0 && rgb.r < 1.0 {
                valid_pairs.push((rgb.r.ln(), xyz.y.ln()));
            }
        }
        if valid_pairs.len() >= 2 {
            let n = valid_pairs.len() as f64;
            let sum_x: f64 = valid_pairs.iter().map(|(x, _)| x).sum();
            let sum_y: f64 = valid_pairs.iter().map(|(_, y)| y).sum();
            let sum_xy: f64 = valid_pairs.iter().map(|(x, y)| x * y).sum();
            let sum_xx: f64 = valid_pairs.iter().map(|(x, _)| x * x).sum();
            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
            if slope.is_finite() && slope > 0.5 && slope < 5.0 {
                gamma_estimate = slope;
            }
        }

        // Compute per-channel correction factors
        let max_y = readings.last().map(|(_, xyz)| xyz.y).unwrap_or(100.0);
        let mut r_corr = Vec::with_capacity(readings.len());
        let mut g_corr = Vec::with_capacity(readings.len());
        let mut b_corr = Vec::with_capacity(readings.len());

        for (rgb, xyz) in readings {
            if xyz.y > 0.0 {
                let target_y = rgb.r.powf(gamma_estimate) * max_y;
                let factor = target_y / xyz.y;
                r_corr.push(factor);
                g_corr.push(factor);
                b_corr.push(factor);
            } else {
                r_corr.push(1.0);
                g_corr.push(1.0);
                b_corr.push(1.0);
            }
        }

        Ok(GreyscaleAnalysis {
            gamma: gamma_estimate,
            max_de,
            avg_de,
            white_balance_errors: errors,
            per_channel_corrections: [r_corr, g_corr, b_corr],
        })
    }
}
