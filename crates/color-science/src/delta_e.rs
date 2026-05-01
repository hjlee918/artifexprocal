//! DeltaE implementations: CIE DE2000, DE76.
//!
//! DE2000 is validated against the Sharma et al. reference dataset.

use crate::types::Lab;

/// CIE DE2000 color difference between two Lab values.
///
/// Reference: Sharma, G., Wu, W., & Dalal, E. N. (2005).
/// "The CIEDE2000 Color-Difference Formula: Implementation Notes,
/// Supplementary Test Data, and Mathematical Observations."
/// Color Research & Application, 30(1), 21–30.
pub fn delta_e_2000(lab1: Lab, lab2: Lab) -> f64 {
    let l1 = lab1.l;
    let a1 = lab1.a;
    let b1 = lab1.b;
    let l2 = lab2.l;
    let a2 = lab2.a;
    let b2 = lab2.b;

    let c1 = (a1.powi(2) + b1.powi(2)).sqrt();
    let c2 = (a2.powi(2) + b2.powi(2)).sqrt();
    let c_avg = (c1 + c2) / 2.0;

    let g = 0.5 * (1.0 - (c_avg.powi(7) / (c_avg.powi(7) + 25.0f64.powi(7))).sqrt());

    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;

    let c1p = (a1p.powi(2) + b1.powi(2)).sqrt();
    let c2p = (a2p.powi(2) + b2.powi(2)).sqrt();

    let h1p = if c1p == 0.0 {
        0.0
    } else {
        let mut h = b1.atan2(a1p).to_degrees();
        if h < 0.0 {
            h += 360.0;
        }
        h
    };
    let h2p = if c2p == 0.0 {
        0.0
    } else {
        let mut h = b2.atan2(a2p).to_degrees();
        if h < 0.0 {
            h += 360.0;
        }
        h
    };

    let delta_lp = l2 - l1;
    let delta_cp = c2p - c1p;

    let delta_hp = if c1p == 0.0 || c2p == 0.0 {
        0.0
    } else {
        let mut dh = h2p - h1p;
        if dh > 180.0 {
            dh -= 360.0;
        } else if dh < -180.0 {
            dh += 360.0;
        }
        dh
    };

    let delta_hp_rad = delta_hp.to_radians();
    let delta_hp_val = 2.0 * (c1p * c2p).sqrt() * (delta_hp_rad / 2.0).sin();

    let l_avg = (l1 + l2) / 2.0;
    let c_avg_p = (c1p + c2p) / 2.0;

    let h_avg_p = if c1p == 0.0 || c2p == 0.0 {
        h1p + h2p
    } else {
        let mut h = (h1p + h2p) / 2.0;
        if (h2p - h1p).abs() > 180.0 {
            if h < 180.0 {
                h += 180.0;
            } else {
                h -= 180.0;
            }
        }
        h
    };

    let t = 1.0 - 0.17 * (h_avg_p - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_avg_p).to_radians().cos()
        + 0.32 * (3.0 * h_avg_p + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_avg_p - 63.0).to_radians().cos();

    let delta_theta = 30.0 * ((-((h_avg_p - 275.0) / 25.0).powi(2))).exp();
    let r_c = 2.0 * (c_avg_p.powi(7) / (c_avg_p.powi(7) + 25.0f64.powi(7))).sqrt();
    let s_l = 1.0 + (0.015 * (l_avg - 50.0).powi(2)) / (20.0 + (l_avg - 50.0).powi(2)).sqrt();
    let s_c = 1.0 + 0.045 * c_avg_p;
    let s_h = 1.0 + 0.015 * c_avg_p * t;
    let r_t = -r_c * (2.0 * delta_theta.to_radians()).sin();

    let l_term = delta_lp / (s_l * 1.0);
    let c_term = delta_cp / (s_c * 1.0);
    let h_term = delta_hp_val / (s_h * 1.0);

    (l_term.powi(2) + c_term.powi(2) + h_term.powi(2) + r_t * c_term * h_term).sqrt()
}

/// CIE DE76 (Euclidean distance in Lab).
pub fn delta_e_76(lab1: Lab, lab2: Lab) -> f64 {
    ((lab2.l - lab1.l).powi(2) + (lab2.a - lab1.a).powi(2) + (lab2.b - lab1.b).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Sharma et al. DE2000 reference dataset ──────────────────
    // Each tuple: (L1, a1, b1, L2, a2, b2, expected_DE2000)
    // Source: Sharma, G., Wu, W., & Dalal, E. N. (2005).
    // "The CIEDE2000 Color-Difference Formula: Implementation Notes,
    // Supplementary Test Data, and Mathematical Observations."
    // Color Research & Application, 30(1), 21-30.
    // Data file: ciede2000testdata.txt (34 cases total).
    // Cases 0-19 below are Pairs 1-20 from the paper.

    #[test]
    fn de2000_sharma_dataset() {
        let cases = [
            (50.0000, 2.6772, -79.7751, 50.0000, 0.0000, -82.7485, 2.0425),
            (50.0000, 3.1571, -77.2803, 50.0000, 0.0000, -82.7485, 2.8615),
            (50.0000, 2.8361, -74.0200, 50.0000, 0.0000, -82.7485, 3.4412),
            (50.0000, -1.3802, -84.2814, 50.0000, 0.0000, -82.7485, 1.0000),
            (50.0000, -1.1848, -84.8006, 50.0000, 0.0000, -82.7485, 1.0000),
            (50.0000, -0.9009, -85.5211, 50.0000, 0.0000, -82.7485, 1.0000),
            (50.0000, 0.0000, 0.0000, 50.0000, -1.0000, 2.0000, 2.3669),
            (50.0000, -1.0000, 2.0000, 50.0000, 0.0000, 0.0000, 2.3669),
            (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0009, 7.1792),
            (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0010, 7.1792),
            (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0011, 7.2195),
            (50.0000, 2.4900, -0.0010, 50.0000, -2.4900, 0.0012, 7.2195),
            (50.0000, -0.0010, 2.4900, 50.0000, 0.0009, -2.4900, 4.8045),
            (50.0000, -0.0010, 2.4900, 50.0000, 0.0010, -2.4900, 4.8045),
            (50.0000, -0.0010, 2.4900, 50.0000, 0.0011, -2.4900, 4.7461),
            (50.0000, -0.0010, 2.4900, 50.0000, 0.0012, -2.4900, 4.7461),
            (50.0000, 2.5000, 0.0000, 50.0000, 0.0000, -2.5000, 4.3065),
            (50.0000, 2.5000, 0.0000, 73.0000, 25.0000, -18.0000, 27.1492),
            (50.0000, 2.5000, 0.0000, 61.0000, -5.0000, 29.0000, 22.8977),
            (50.0000, 2.5000, 0.0000, 56.0000, -27.0000, -3.0000, 31.9030),
            (50.0000, 2.5000, 0.0000, 58.0000, 24.0000, 15.0000, 19.4535),
            (50.0000, 2.5000, 0.0000, 50.0000, 3.1736, 0.5854, 1.0000),
            (50.0000, 2.5000, 0.0000, 50.0000, 3.2972, 0.0000, 1.0000),
            (50.0000, 2.5000, 0.0000, 50.0000, 1.8634, 0.5757, 1.0000),
            (50.0000, 2.5000, 0.0000, 50.0000, 3.2592, 0.3350, 1.0000),
            (60.2574, -34.0099, 36.2677, 60.4626, -34.1751, 39.4387, 1.2644),
            (63.0109, -31.0961, -5.8663, 62.8187, -29.7946, -4.0864, 1.2630),
            (61.2901, 3.7196, -5.3901, 61.4292, 2.2480, -4.9620, 1.8731),
            (35.0831, -44.1164, 3.7933, 35.0232, -40.0716, 1.5901, 1.8645),
            (22.7233, 20.0904, -46.6940, 23.0331, 14.9730, -42.5619, 2.0373),
            (36.4612, 47.8580, 18.3852, 36.2715, 50.5065, 21.2231, 1.4146),
            (90.8027, -2.0831, 1.4410, 91.1528, -1.6435, 0.0447, 1.4441),
            (90.9257, -0.5406, -0.9208, 88.6381, -0.8985, -0.7239, 1.5381),
            (6.7747, -0.2908, -2.4247, 5.8714, -0.0985, -2.2286, 0.6377),
            (2.0776, 0.0795, -1.1350, 0.9033, -0.0636, -0.5514, 0.9082),
        ];

        for (l1, a1, b1, l2, a2, b2, expected) in cases {
            let lab1 = Lab { l: l1, a: a1, b: b1 };
            let lab2 = Lab { l: l2, a: a2, b: b2 };
            let de = delta_e_2000(lab1, lab2);
            assert!(
                (de - expected).abs() < 0.001,
                "DE2000 for ({},{},{}) vs ({},{},{}) expected {} got {}",
                l1, a1, b1, l2, a2, b2, expected, de
            );
        }
    }

    #[test]
    fn de76_identity_is_zero() {
        let lab = Lab { l: 50.0, a: 20.0, b: -30.0 };
        assert!(delta_e_76(lab, lab).abs() < 1e-10);
    }

    #[test]
    fn de76_symmetric() {
        let a = Lab { l: 50.0, a: 10.0, b: 10.0 };
        let b = Lab { l: 60.0, a: 20.0, b: 20.0 };
        assert!((delta_e_76(a, b) - delta_e_76(b, a)).abs() < 1e-10);
    }
}
