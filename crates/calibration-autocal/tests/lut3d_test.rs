use calibration_autocal::lut3d::TetrahedralInterpolator;
use hal::types::{Lut3D, RGB};

fn identity_lut(size: usize) -> Lut3D {
    let mut data = Vec::with_capacity(size * size * size);
    for r in 0..size {
        for g in 0..size {
            for b in 0..size {
                let rf = r as f64 / (size - 1) as f64;
                let gf = g as f64 / (size - 1) as f64;
                let bf = b as f64 / (size - 1) as f64;
                data.push(RGB { r: rf, g: gf, b: bf });
            }
        }
    }
    Lut3D { data, size }
}

#[test]
fn tetrahedral_identity_corner_values() {
    let lut = identity_lut(5);
    let interp = TetrahedralInterpolator::new(lut);

    let c = interp.lookup(0.0, 0.0, 0.0);
    assert!((c.r - 0.0).abs() < 0.001);
    assert!((c.g - 0.0).abs() < 0.001);
    assert!((c.b - 0.0).abs() < 0.001);

    let c = interp.lookup(1.0, 1.0, 1.0);
    assert!((c.r - 1.0).abs() < 0.001);
    assert!((c.g - 1.0).abs() < 0.001);
    assert!((c.b - 1.0).abs() < 0.001);
}

#[test]
fn tetrahedral_identity_center() {
    let lut = identity_lut(5);
    let interp = TetrahedralInterpolator::new(lut);
    let c = interp.lookup(0.5, 0.5, 0.5);
    assert!((c.r - 0.5).abs() < 0.02);
    assert!((c.g - 0.5).abs() < 0.02);
    assert!((c.b - 0.5).abs() < 0.02);
}

#[test]
fn tetrahedral_off_grid_interpolation() {
    let lut = identity_lut(5);
    let interp = TetrahedralInterpolator::new(lut);

    // Off-grid point where dr, dg, db are all non-zero and distinct
    let c = interp.lookup(0.1, 0.2, 0.3);
    assert!((c.r - 0.1).abs() < 0.001, "Expected r ~ 0.1, got {}", c.r);
    assert!((c.g - 0.2).abs() < 0.001, "Expected g ~ 0.2, got {}", c.g);
    assert!((c.b - 0.3).abs() < 0.001, "Expected b ~ 0.3, got {}", c.b);
}

#[test]
fn tetrahedral_scaled_lut() {
    let size = 3;
    let mut data = Vec::with_capacity(size * size * size);
    for r in 0..size {
        for g in 0..size {
            for b in 0..size {
                let rf = r as f64 / (size - 1) as f64;
                let gf = g as f64 / (size - 1) as f64;
                let bf = b as f64 / (size - 1) as f64;
                data.push(RGB {
                    r: rf * 2.0,
                    g: gf * 2.0,
                    b: bf * 2.0,
                });
            }
        }
    }
    let lut = Lut3D { data, size };
    let interp = TetrahedralInterpolator::new(lut);

    let c = interp.lookup(0.5, 0.5, 0.5);
    assert!(
        (c.r - 1.0).abs() < 0.05,
        "Expected r ~ 1.0, got {}",
        c.r
    );
    assert!(
        (c.g - 1.0).abs() < 0.05,
        "Expected g ~ 1.0, got {}",
        c.g
    );
    assert!(
        (c.b - 1.0).abs() < 0.05,
        "Expected b ~ 1.0, got {}",
        c.b
    );
}
