use color_science::delta_e::delta_e_2000;
use color_science::types::{XYZ, WhitePoint};

/// A 3×3 correction matrix that transforms colorimeter XYZ readings
/// to better match spectrophotometer reference readings.
#[derive(Debug, Clone, PartialEq)]
pub struct CorrectionMatrix {
    pub m: [[f64; 3]; 3],
}

impl CorrectionMatrix {
    /// Identity matrix (no correction).
    pub fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    /// Apply the correction matrix to an XYZ reading.
    pub fn apply(&self, xyz: &XYZ) -> XYZ {
        XYZ {
            x: self.m[0][0] * xyz.x + self.m[0][1] * xyz.y + self.m[0][2] * xyz.z,
            y: self.m[1][0] * xyz.x + self.m[1][1] * xyz.y + self.m[1][2] * xyz.z,
            z: self.m[2][0] * xyz.x + self.m[2][1] * xyz.y + self.m[2][2] * xyz.z,
        }
    }

    /// Compute average dE2000 between corrected colorimeter and reference spectro readings.
    pub fn accuracy(&self, colorimeter: &[XYZ], reference: &[XYZ]) -> f64 {
        if colorimeter.len() != reference.len() || colorimeter.is_empty() {
            return f64::MAX;
        }
        let wp = WhitePoint::D65;
        let total_de: f64 = colorimeter
            .iter()
            .zip(reference.iter())
            .map(|(c, r)| {
                let corrected = self.apply(c);
                let c_lab = corrected.to_lab(wp);
                let r_lab = r.to_lab(wp);
                delta_e_2000(&c_lab, &r_lab)
            })
            .sum();
        total_de / colorimeter.len() as f64
    }

    /// Compute max dE2000 between corrected colorimeter and reference spectro readings.
    pub fn max_error(&self, colorimeter: &[XYZ], reference: &[XYZ]) -> f64 {
        if colorimeter.len() != reference.len() || colorimeter.is_empty() {
            return f64::MAX;
        }
        let wp = WhitePoint::D65;
        colorimeter
            .iter()
            .zip(reference.iter())
            .map(|(c, r)| {
                let corrected = self.apply(c);
                let c_lab = corrected.to_lab(wp);
                let r_lab = r.to_lab(wp);
                delta_e_2000(&c_lab, &r_lab)
            })
            .fold(0.0, f64::max)
    }
}

/// Generate a correction matrix using least-squares fitting.
///
/// For each patch `i`, we want:
///   M * colorimeter_i ≈ reference_i
///
/// This is solved as three independent linear least-squares problems
/// (one per output channel) using the normal equations.
pub fn generate_correction_matrix(
    colorimeter_xyz: &[XYZ],
    reference_xyz: &[XYZ],
) -> Result<CorrectionMatrix, ProfilingError> {
    if colorimeter_xyz.len() != reference_xyz.len() {
        return Err(ProfilingError::MismatchedLengths);
    }
    let n = colorimeter_xyz.len();
    if n < 3 {
        return Err(ProfilingError::InsufficientData {
            got: n,
            need: 3,
        });
    }

    // Build C^T * C (3×3) and C^T * s_j for each channel j
    // C is n×3 where row i = [cix, ciy, ciz]
    let mut ctc = [[0.0; 3]; 3];
    let mut cts = [0.0; 3]; // for x channel
    let mut cty = [0.0; 3]; // for y channel
    let mut ctz = [0.0; 3]; // for z channel

    for (c, r) in colorimeter_xyz.iter().zip(reference_xyz.iter()) {
        let ci = [c.x, c.y, c.z];
        let ri = [r.x, r.y, r.z];

        for i in 0..3 {
            for j in 0..3 {
                ctc[i][j] += ci[i] * ci[j];
            }
            cts[i] += ci[i] * ri[0];
            cty[i] += ci[i] * ri[1];
            ctz[i] += ci[i] * ri[2];
        }
    }

    // Solve three 3×3 systems: ctc * m_j = rhs_j
    let mx = solve_3x3(ctc, cts).ok_or(ProfilingError::SingularMatrix)?;
    let my = solve_3x3(ctc, cty).ok_or(ProfilingError::SingularMatrix)?;
    let mz = solve_3x3(ctc, ctz).ok_or(ProfilingError::SingularMatrix)?;

    Ok(CorrectionMatrix {
        m: [mx, my, mz],
    })
}

/// Errors that can occur during profiling.
#[derive(Debug, Clone, PartialEq)]
pub enum ProfilingError {
    MismatchedLengths,
    InsufficientData { got: usize, need: usize },
    SingularMatrix,
}

impl std::fmt::Display for ProfilingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfilingError::MismatchedLengths => {
                write!(f, "Colorimeter and reference data have different lengths")
            }
            ProfilingError::InsufficientData { got, need } => {
                write!(f, "Insufficient data points: got {got}, need at least {need}")
            }
            ProfilingError::SingularMatrix => {
                write!(f, "Could not solve for correction matrix (singular system)")
            }
        }
    }
}

impl std::error::Error for ProfilingError {}

/// Solve a 3×3 linear system A * x = b using Cramer's rule.
fn solve_3x3(a: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let det = det_3x3(a);
    if det.abs() < 1e-12 {
        return None;
    }

    let mut ax = a;
    ax[0][0] = b[0];
    ax[1][0] = b[1];
    ax[2][0] = b[2];

    let mut ay = a;
    ay[0][1] = b[0];
    ay[1][1] = b[1];
    ay[2][1] = b[2];

    let mut az = a;
    az[0][2] = b[0];
    az[1][2] = b[1];
    az[2][2] = b[2];

    Some([
        det_3x3(ax) / det,
        det_3x3(ay) / det,
        det_3x3(az) / det,
    ])
}

fn det_3x3(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Export correction matrix to Argyll `.ccmx` XML format.
pub fn export_ccmx<W: std::io::Write>(
    matrix: &CorrectionMatrix,
    meter_name: &str,
    reference_name: &str,
    writer: &mut W,
) -> std::io::Result<()> {
    writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(writer, "<ccmx>")?;
    writeln!(writer, "  <instrument_make>{}</instrument_make>", escape_xml(meter_name))?;
    writeln!(writer, "  <instrument_model>{}</instrument_model>", escape_xml(meter_name))?;
    writeln!(writer, "  <reference_instrument_make>{}</reference_instrument_make>", escape_xml(reference_name))?;
    writeln!(writer, "  <reference_instrument_model>{}</reference_instrument_model>", escape_xml(reference_name))?;
    writeln!(writer, "  <created>{}</created>", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"))?;
    writeln!(writer, "  <matrix>")?;
    for row in &matrix.m {
        writeln!(
            writer,
            "    <row>{}</row>",
            row.iter().map(|v| format!("{:.6}", v)).collect::<Vec<_>>().join(" ")
        )?;
    }
    writeln!(writer, "  </matrix>")?;
    writeln!(writer, "</ccmx>")?;
    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_matrix() {
        let identity = CorrectionMatrix::identity();
        let xyz = XYZ { x: 50.0, y: 100.0, z: 25.0 };
        let corrected = identity.apply(&xyz);
        assert!((corrected.x - 50.0).abs() < 1e-6);
        assert!((corrected.y - 100.0).abs() < 1e-6);
        assert!((corrected.z - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_least_squares_perfect_case() {
        // If colorimeter == reference, matrix should be identity
        // Use diverse XYZ values that span 3D space (not collinear)
        let colorimeter = vec![
            XYZ { x: 10.0, y: 20.0, z: 5.0 },
            XYZ { x: 5.0, y: 40.0, z: 15.0 },
            XYZ { x: 50.0, y: 10.0, z: 25.0 },
            XYZ { x: 70.0, y: 80.0, z: 35.0 },
        ];
        let reference = colorimeter.clone();

        let matrix = generate_correction_matrix(&colorimeter, &reference).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (matrix.m[i][j] - expected).abs() < 1e-5,
                    "m[{i}][{j}] = {} != {expected}",
                    matrix.m[i][j]
                );
            }
        }
    }

    #[test]
    fn test_least_squares_scaled_case() {
        // If reference = 2 * colorimeter, matrix should be 2*I
        let colorimeter = vec![
            XYZ { x: 10.0, y: 20.0, z: 5.0 },
            XYZ { x: 5.0, y: 40.0, z: 15.0 },
            XYZ { x: 50.0, y: 10.0, z: 25.0 },
            XYZ { x: 70.0, y: 80.0, z: 35.0 },
        ];
        let reference: Vec<XYZ> = colorimeter
            .iter()
            .map(|c| XYZ { x: c.x * 2.0, y: c.y * 2.0, z: c.z * 2.0 })
            .collect();

        let matrix = generate_correction_matrix(&colorimeter, &reference).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 2.0 } else { 0.0 };
                assert!(
                    (matrix.m[i][j] - expected).abs() < 1e-5,
                    "m[{i}][{j}] = {} != {expected}",
                    matrix.m[i][j]
                );
            }
        }
    }

    #[test]
    fn test_accuracy_computation() {
        let identity = CorrectionMatrix::identity();
        let colorimeter = vec![
            XYZ { x: 10.0, y: 20.0, z: 5.0 },
            XYZ { x: 30.0, y: 40.0, z: 15.0 },
        ];
        let reference = colorimeter.clone();
        let acc = identity.accuracy(&colorimeter, &reference);
        assert!(acc < 1e-6);
    }

    #[test]
    fn test_insufficient_data_error() {
        let c = vec![XYZ { x: 1.0, y: 2.0, z: 3.0 }];
        let r = vec![XYZ { x: 1.0, y: 2.0, z: 3.0 }];
        let result = generate_correction_matrix(&c, &r);
        assert!(matches!(result, Err(ProfilingError::InsufficientData { got: 1, need: 3 })));
    }

    #[test]
    fn test_mismatched_lengths_error() {
        let c = vec![XYZ { x: 1.0, y: 2.0, z: 3.0 }; 3];
        let r = vec![XYZ { x: 1.0, y: 2.0, z: 3.0 }; 2];
        let result = generate_correction_matrix(&c, &r);
        assert!(matches!(result, Err(ProfilingError::MismatchedLengths)));
    }

    #[test]
    fn test_export_ccmx_structure() {
        let matrix = CorrectionMatrix::identity();
        let mut buf = Vec::new();
        export_ccmx(&matrix, "i1 Display Pro", "i1 Pro 2", &mut buf).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("<ccmx>"));
        assert!(xml.contains("</ccmx>"));
        assert!(xml.contains("i1 Display Pro"));
        assert!(xml.contains("i1 Pro 2"));
        assert!(xml.contains("<matrix>"));
        assert!(xml.contains("1.000000"));
    }
}
