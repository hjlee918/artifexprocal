//! Blackbody radiator SPD generation and Planckian locus synthesis.
//!
//! Uses Planck's law to generate spectral power distributions, then
//! integrates against the CIE 1931 2° observer to produce XYZ.
//! This is the engine behind FakeMeter's Planckian sweep mode.

use crate::cie1931::{integrate_spd_to_xyz, CIE_WAVELENGTHS_NM};

const H: f64 = 6.62607015e-34;      // Planck constant, J·s
const C: f64 = 2.99792458e8;        // Speed of light, m/s
const K_B: f64 = 1.380649e-23;      // Boltzmann constant, J/K
const C1: f64 = 2.0 * std::f64::consts::PI * H * C * C; // 1st radiation constant
const C2: f64 = H * C / K_B;        // 2nd radiation constant, m·K

/// Generate a blackbody SPD at temperature `cct` (Kelvin).
/// Returns a Vec of spectral radiance values (W·sr⁻¹·m⁻³) at 5 nm intervals
/// from 360–830 nm, matching `CIE_WAVELENGTHS_NM`.
pub fn blackbody_spd(cct: f64) -> Vec<f64> {
    CIE_WAVELENGTHS_NM
        .iter()
        .map(|&wl_nm| {
            let wl = wl_nm * 1e-9; // convert nm → m
            let term = C2 / (wl * cct);
            // Planck's law (spectral radiance per unit wavelength)
            C1 / (wl.powi(5) * (term.exp() - 1.0))
        })
        .collect()
}

/// Synthesize XYZ from a blackbody at given CCT and target luminance (Y in cd/m²).
///
/// This is the same calculation path used by FakeMeter's Planckian sweep.
pub fn blackbody_xyz(cct: f64, target_luminance: f64) -> crate::types::Xyz {
    let spd = blackbody_spd(cct);
    let xyz = integrate_spd_to_xyz(&spd);

    // Scale to target luminance
    if xyz.y == 0.0 {
        return crate::types::Xyz { x: 0.0, y: 0.0, z: 0.0 };
    }
    let scale = target_luminance / xyz.y;
    crate::types::Xyz {
        x: xyz.x * scale,
        y: xyz.y * scale,
        z: xyz.z * scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Xyz;

    #[test]
    fn blackbody_d65_approximates_reference() {
        // Synthesize D65 (~6504K) and verify it lands close to the tabulated D65 XYZ.
        let xyz = blackbody_xyz(6504.0, 100.0);
        let d65_ref = Xyz { x: 95.047, y: 100.0, z: 108.883 };

        // Tolerance is generous because our tabulation is 5 nm and the
        // CIE D65 is an illuminant, not a true blackbody.
        let dx = (xyz.x - d65_ref.x).abs() / d65_ref.x;
        let dy = (xyz.y - d65_ref.y).abs() / d65_ref.y;
        let dz = (xyz.z - d65_ref.z).abs() / d65_ref.z;

        assert!(dx < 0.05, "X mismatch: {} vs {}", xyz.x, d65_ref.x);
        assert!(dy < 0.05, "Y mismatch: {} vs {}", xyz.y, d65_ref.y);
        assert!(dz < 0.05, "Z mismatch: {} vs {}", xyz.z, d65_ref.z);
    }

    #[test]
    fn blackbody_spd_peak_shifts_with_temperature() {
        // Wien's displacement law: peak wavelength λ_max = b / T
        // For 4000K: ~724 nm (visible/red)
        // For 6500K: ~446 nm (visible/blue)
        let spd_4000 = blackbody_spd(4000.0);
        let spd_6500 = blackbody_spd(6500.0);

        let peak_4000 = find_peak_wavelength(&spd_4000);
        let peak_6500 = find_peak_wavelength(&spd_6500);

        // Wien's constant b ≈ 2.898e-3 m·K
        let expected_4000 = 2.898e-3 / 4000.0 * 1e9; // nm
        let expected_6500 = 2.898e-3 / 6500.0 * 1e9; // nm

        assert!((peak_4000 - expected_4000).abs() < 50.0);
        assert!((peak_6500 - expected_6500).abs() < 50.0);
    }

    fn find_peak_wavelength(spd: &[f64]) -> f64 {
        let mut max_val = 0.0;
        let mut max_idx = 0;
        for (i, &v) in spd.iter().enumerate() {
            if v > max_val {
                max_val = v;
                max_idx = i;
            }
        }
        CIE_WAVELENGTHS_NM[max_idx]
    }
}
