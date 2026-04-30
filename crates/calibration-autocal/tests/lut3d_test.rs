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

use calibration_autocal::lut3d::Lut3DEngine;
use calibration_core::state::TargetSpace;
use color_science::types::XYZ;

#[test]
fn lut3d_engine_identity_display() {
    // For a "perfect" display, target RGB = measured XYZ (in normalized form)
    let patches: Vec<(RGB, XYZ)> = vec![
        (RGB { r: 0.0, g: 0.0, b: 0.0 }, XYZ { x: 0.0, y: 0.0, z: 0.0 }),
        (RGB { r: 1.0, g: 0.0, b: 0.0 }, XYZ { x: 100.0, y: 0.0, z: 0.0 }),
        (RGB { r: 0.0, g: 1.0, b: 0.0 }, XYZ { x: 0.0, y: 100.0, z: 0.0 }),
        (RGB { r: 0.0, g: 0.0, b: 1.0 }, XYZ { x: 0.0, y: 0.0, z: 100.0 }),
        (RGB { r: 1.0, g: 1.0, b: 1.0 }, XYZ { x: 100.0, y: 100.0, z: 100.0 }),
        (RGB { r: 0.5, g: 0.5, b: 0.5 }, XYZ { x: 50.0, y: 50.0, z: 50.0 }),
    ];

    let lut = Lut3DEngine::compute(&patches, 5, &TargetSpace::Bt709).unwrap();
    assert_eq!(lut.size, 5);
    assert_eq!(lut.data.len(), 125);

    // Black (0,0,0) at index 0 should be approximately (0,0,0)
    let black = lut.data[0];
    assert!(
        (black.r - 0.0).abs() < 0.2,
        "Black r should be ~0.0, got {}",
        black.r
    );
    assert!(
        (black.g - 0.0).abs() < 0.2,
        "Black g should be ~0.0, got {}",
        black.g
    );
    assert!(
        (black.b - 0.0).abs() < 0.2,
        "Black b should be ~0.0, got {}",
        black.b
    );

    // White point (1,1,1) at index 124 should be approximately (1,1,1)
    let white = lut.data[124]; // (4,4,4) = (4*5+4)*5+4 = 124
    assert!(
        (white.r - 1.0).abs() < 0.2,
        "White r should be ~1.0, got {}",
        white.r
    );
    assert!(
        (white.g - 1.0).abs() < 0.2,
        "White g should be ~1.0, got {}",
        white.g
    );
    assert!(
        (white.b - 1.0).abs() < 0.2,
        "White b should be ~1.0, got {}",
        white.b
    );

    // Mid-gray (0.5,0.5,0.5) at index (2*5+2)*5+2 = 62
    let gray = lut.data[62];
    assert!(
        (gray.r - 0.5).abs() < 0.2,
        "Gray r should be ~0.5, got {}",
        gray.r
    );
    assert!(
        (gray.g - 0.5).abs() < 0.2,
        "Gray g should be ~0.5, got {}",
        gray.g
    );
    assert!(
        (gray.b - 0.5).abs() < 0.2,
        "Gray b should be ~0.5, got {}",
        gray.b
    );

    // Red primary (1,0,0) at index (4*5+0)*5+0 = 100
    let red = lut.data[100];
    assert!(
        (red.r - 1.0).abs() < 0.2,
        "Red r should be ~1.0, got {}",
        red.r
    );
    assert!(
        red.g.abs() < 0.2,
        "Red g should be ~0.0, got {}",
        red.g
    );
    assert!(
        red.b.abs() < 0.2,
        "Red b should be ~0.0, got {}",
        red.b
    );
}

#[test]
fn lut3d_engine_empty_patches_fails() {
    let result = Lut3DEngine::compute(&[], 5, &TargetSpace::Bt709);
    assert!(result.is_err());
}

#[test]
fn lut3d_engine_size_too_small_fails() {
    let patches = vec![(RGB { r: 0.5, g: 0.5, b: 0.5 }, XYZ { x: 50.0, y: 50.0, z: 50.0 })];
    let result = Lut3DEngine::compute(&patches, 1, &TargetSpace::Bt709);
    assert!(result.is_err());
}

fn make_uniform_lut(size: usize, value: f64) -> Lut3D {
    let data = vec![RGB { r: value, g: value, b: value }; size * size * size];
    Lut3D { data, size }
}

#[test]
fn downsample_uniform_lut() {
    let lut33 = make_uniform_lut(33, 0.5);
    let lut17 = Lut3DEngine::downsample_33_to_17(&lut33).unwrap();
    assert_eq!(lut17.size, 17);
    assert_eq!(lut17.data.len(), 4913); // 17³

    for rgb in &lut17.data {
        assert!((rgb.r - 0.5).abs() < 0.001);
        assert!((rgb.g - 0.5).abs() < 0.001);
        assert!((rgb.b - 0.5).abs() < 0.001);
    }
}

#[test]
fn downsample_wrong_size_fails() {
    let lut5 = make_uniform_lut(5, 0.5);
    let result = Lut3DEngine::downsample_33_to_17(&lut5);
    assert!(result.is_err());
}
