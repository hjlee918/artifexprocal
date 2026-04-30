use crate::types::Lab;
use std::f64::consts::PI;

/// CIEDE2000 color difference metric.
/// Reference: Sharma et al. (2005) "The CIEDE2000 Color-Difference Formula"
pub fn delta_e_2000(lab1: &Lab, lab2: &Lab) -> f64 {
    const KL: f64 = 1.0;
    const KC: f64 = 1.0;
    const KH: f64 = 1.0;

    let l1 = lab1.L;
    let a1 = lab1.a;
    let b1 = lab1.b;
    let l2 = lab2.L;
    let a2 = lab2.a;
    let b2 = lab2.b;

    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();

    let c_avg = (c1 + c2) / 2.0;
    let g = 0.5 * (1.0 - (c_avg.powi(7) / (c_avg.powi(7) + 25f64.powi(7))).sqrt());

    let a1_prime = a1 * (1.0 + g);
    let a2_prime = a2 * (1.0 + g);

    let c1_prime = (a1_prime * a1_prime + b1 * b1).sqrt();
    let c2_prime = (a2_prime * a2_prime + b2 * b2).sqrt();

    let h1_prime = h_prime(a1_prime, b1);
    let h2_prime = h_prime(a2_prime, b2);

    let delta_l_prime = l2 - l1;
    let delta_c_prime = c2_prime - c1_prime;

    let delta_h_raw = delta_h(c1_prime, c2_prime, h1_prime, h2_prime);
    let delta_h_prime = 2.0 * (c1_prime * c2_prime).sqrt() * (delta_h_raw.to_radians() / 2.0).sin();

    let l_avg = (l1 + l2) / 2.0;
    let c_avg_prime = (c1_prime + c2_prime) / 2.0;

    let h_avg_prime = h_avg(c1_prime, c2_prime, h1_prime, h2_prime);

    let t = 1.0
        - 0.17 * ((h_avg_prime - 30.0) * PI / 180.0).cos()
        + 0.24 * ((2.0 * h_avg_prime) * PI / 180.0).cos()
        + 0.32 * ((3.0 * h_avg_prime + 6.0) * PI / 180.0).cos()
        - 0.20 * ((4.0 * h_avg_prime - 63.0) * PI / 180.0).cos();

    let delta_theta = 30.0 * (-((h_avg_prime - 275.0) / 25.0).powi(2)).exp();

    let rc = 2.0 * (c_avg_prime.powi(7) / (c_avg_prime.powi(7) + 25f64.powi(7))).sqrt();

    let sl = 1.0 + (0.015 * (l_avg - 50.0).powi(2)) / (20.0 + (l_avg - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * c_avg_prime;
    let sh = 1.0 + 0.015 * c_avg_prime * t;

    let rt = -(2.0 * delta_theta.to_radians()).sin() * rc;

    let term1 = delta_l_prime / (KL * sl);
    let term2 = delta_c_prime / (KC * sc);
    let term3 = delta_h_prime / (KH * sh);

    (term1 * term1 + term2 * term2 + term3 * term3 + rt * term2 * term3).sqrt()
}

fn h_prime(a: f64, b: f64) -> f64 {
    if a == 0.0 && b == 0.0 {
        0.0
    } else {
        let h = b.atan2(a).to_degrees();
        if h < 0.0 {
            h + 360.0
        } else {
            h
        }
    }
}

fn delta_h(c1: f64, c2: f64, h1: f64, h2: f64) -> f64 {
    if c1 == 0.0 || c2 == 0.0 {
        0.0
    } else {
        let dh = h2 - h1;
        if dh.abs() <= 180.0 {
            dh
        } else if h2 <= h1 {
            dh + 360.0
        } else {
            dh - 360.0
        }
    }
}

fn h_avg(c1: f64, c2: f64, h1: f64, h2: f64) -> f64 {
    if c1 == 0.0 && c2 == 0.0 {
        0.0
    } else if c1 == 0.0 {
        h2
    } else if c2 == 0.0 {
        h1
    } else {
        let sum = h1 + h2;
        let dh = (h2 - h1).abs();
        if dh <= 180.0 {
            sum / 2.0
        } else if sum < 360.0 {
            (sum + 360.0) / 2.0
        } else {
            (sum - 360.0) / 2.0
        }
    }
}
