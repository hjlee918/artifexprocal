use color_science::types::*;
use color_science::conversion::*;

#[test]
fn test_xyz_creation() {
    let xyz = XYZ { x: 95.047, y: 100.0, z: 108.883 };
    assert_eq!(xyz.x, 95.047);
    assert_eq!(xyz.y, 100.0);
    assert_eq!(xyz.z, 108.883);
}

#[test]
fn test_xyy_creation() {
    let xyy = XyY { x: 0.3127, y: 0.3290, Y: 100.0 };
    assert_eq!(xyy.x, 0.3127);
    assert_eq!(xyy.y, 0.3290);
    assert_eq!(xyy.Y, 100.0);
}

#[test]
fn test_lab_creation() {
    let lab = Lab { L: 53.2329, a: 80.1093, b: 67.2201 };
    assert_eq!(lab.L, 53.2329);
    assert_eq!(lab.a, 80.1093);
    assert_eq!(lab.b, 67.2201);
}

#[test]
fn test_rgb_creation() {
    let rgb = RGB { r: 1.0, g: 0.0, b: 0.0 };
    assert_eq!(rgb.r, 1.0);
    assert_eq!(rgb.g, 0.0);
    assert_eq!(rgb.b, 0.0);
}

#[test]
fn test_white_point_d65() {
    let wp = WhitePoint::D65;
    let xyz = wp.to_xyz();
    assert!((xyz.x - 95.047).abs() < 0.001);
    assert!((xyz.y - 100.0).abs() < 0.001);
    assert!((xyz.z - 108.883).abs() < 0.001);
}

#[test]
fn test_xyz_to_xyy_srgb_red() {
    let xyz = XYZ { x: 41.2456, y: 21.2673, z: 1.9334 };
    let xyy = xyz.to_xyy();
    assert!((xyy.x - 0.6399).abs() < 0.0001);
    assert!((xyy.y - 0.3300).abs() < 0.0001);
    assert!((xyy.Y - 21.2673).abs() < 0.0001);
}

#[test]
fn test_xyy_to_xyz_srgb_red() {
    let xyy = XyY { x: 0.6399, y: 0.3300, Y: 21.2673 };
    let xyz = xyy.to_xyz();
    assert!((xyz.x - 41.2456).abs() < 0.01);
    assert!((xyz.y - 21.2673).abs() < 0.01);
    assert!((xyz.z - 1.9334).abs() < 0.01);
}

#[test]
fn test_xyz_to_xyy_zero_returns_zero() {
    let xyz = XYZ { x: 0.0, y: 0.0, z: 0.0 };
    let xyy = xyz.to_xyy();
    assert_eq!(xyy.x, 0.0);
    assert_eq!(xyy.y, 0.0);
    assert_eq!(xyy.Y, 0.0);
}

#[test]
fn test_xyz_xyy_roundtrip() {
    let original = XYZ { x: 50.0, y: 75.0, z: 25.0 };
    let xyy = original.to_xyy();
    let back = xyy.to_xyz();
    assert!((original.x - back.x).abs() < 0.0001);
    assert!((original.y - back.y).abs() < 0.0001);
    assert!((original.z - back.z).abs() < 0.0001);
}

#[test]
fn test_xyz_to_lab_srgb_red() {
    let xyz = XYZ { x: 41.2456, y: 21.2673, z: 1.9334 };
    let lab = xyz.to_lab(WhitePoint::D65);
    assert!((lab.L - 53.2329).abs() < 0.03);
    assert!((lab.a - 80.1093).abs() < 0.03);
    assert!((lab.b - 67.2201).abs() < 0.03);
}

#[test]
fn test_xyz_to_lab_d65_white() {
    let xyz = XYZ { x: 95.047, y: 100.0, z: 108.883 };
    let lab = xyz.to_lab(WhitePoint::D65);
    assert!((lab.L - 100.0).abs() < 0.01);
    assert!(lab.a.abs() < 0.01);
    assert!(lab.b.abs() < 0.01);
}

#[test]
fn test_lab_to_xyz_srgb_red() {
    let lab = Lab { L: 53.2329, a: 80.1093, b: 67.2201 };
    let xyz = lab.to_xyz(WhitePoint::D65);
    assert!((xyz.x - 41.2456).abs() < 0.03);
    assert!((xyz.y - 21.2673).abs() < 0.03);
    assert!((xyz.z - 1.9334).abs() < 0.03);
}

#[test]
fn test_xyz_lab_roundtrip() {
    let original = XYZ { x: 50.0, y: 75.0, z: 25.0 };
    let lab = original.to_lab(WhitePoint::D65);
    let back = lab.to_xyz(WhitePoint::D65);
    assert!((original.x - back.x).abs() < 0.001);
    assert!((original.y - back.y).abs() < 0.001);
    assert!((original.z - back.z).abs() < 0.001);
}

use color_science::delta_e::*;

#[test]
fn test_delta_e_2000_sharma_pair1() {
    let lab1 = Lab { L: 50.0000, a: 2.6772, b: -79.7751 };
    let lab2 = Lab { L: 50.0000, a: 0.0000, b: -82.7485 };
    let de = delta_e_2000(&lab1, &lab2);
    assert!((de - 2.0425).abs() < 0.0001);
}

#[test]
fn test_delta_e_2000_sharma_pair2() {
    let lab1 = Lab { L: 50.0000, a: -1.1848, b: -84.8006 };
    let lab2 = Lab { L: 50.0000, a: 0.0000, b: -82.7485 };
    let de = delta_e_2000(&lab1, &lab2);
    assert!((de - 1.0000).abs() < 0.0001);
}

#[test]
fn test_delta_e_2000_sharma_pair24() {
    let lab1 = Lab { L: 60.2574, a: -34.0099, b: 36.2677 };
    let lab2 = Lab { L: 60.4626, a: -34.1751, b: 39.4387 };
    let de = delta_e_2000(&lab1, &lab2);
    assert!((de - 1.2644).abs() < 0.0001);
}

#[test]
fn test_delta_e_2000_identical_colors() {
    let lab = Lab { L: 50.0, a: 10.0, b: -20.0 };
    let de = delta_e_2000(&lab, &lab);
    assert!(de.abs() < 0.0001);
}

#[test]
fn test_delta_e_2000_large_difference() {
    let lab1 = Lab { L: 50.0, a: 2.5, b: 0.0 };
    let lab2 = Lab { L: 73.0, a: 25.0, b: -18.0 };
    let de = delta_e_2000(&lab1, &lab2);
    assert!((de - 27.1492).abs() < 0.0001);
}

use color_science::adaptation::*;

#[test]
fn test_bradford_d65_to_d50() {
    let xyz_d65 = XYZ { x: 95.047, y: 100.0, z: 108.883 };
    let xyz_d50 = bradford_adapt(&xyz_d65, WhitePoint::D65, WhitePoint::D50);
    assert!((xyz_d50.x - 96.421).abs() < 0.1);
    assert!((xyz_d50.y - 100.0).abs() < 0.1);
    assert!((xyz_d50.z - 82.519).abs() < 0.1);
}

#[test]
fn test_bradford_same_whitepoint_no_change() {
    let xyz = XYZ { x: 50.0, y: 75.0, z: 25.0 };
    let adapted = bradford_adapt(&xyz, WhitePoint::D65, WhitePoint::D65);
    assert!((xyz.x - adapted.x).abs() < 0.0001);
    assert!((xyz.y - adapted.y).abs() < 0.0001);
    assert!((xyz.z - adapted.z).abs() < 0.0001);
}

use color_science::gamma::*;

#[test]
fn test_srgb_gamma_encode_zero() {
    assert!((srgb_gamma_encode(0.0) - 0.0).abs() < 0.0001);
}

#[test]
fn test_srgb_gamma_encode_one() {
    assert!((srgb_gamma_encode(1.0) - 1.0).abs() < 0.0001);
}

#[test]
fn test_srgb_gamma_roundtrip() {
    let linear = 0.5;
    let encoded = srgb_gamma_encode(linear);
    let decoded = srgb_gamma_decode(encoded);
    assert!((linear - decoded).abs() < 0.00001);
}

#[test]
fn test_gamma_22() {
    let linear = 0.5;
    let encoded = gamma_encode(linear, 2.2);
    let expected = linear.powf(1.0 / 2.2);
    assert!((encoded - expected).abs() < 0.0001);
}

#[test]
fn test_pq_encode_decode() {
    let linear = 0.1;
    let encoded = pq_encode(linear);
    let decoded = pq_decode(encoded);
    assert!((linear - decoded).abs() < 0.00001);
}

#[test]
fn test_hlg_encode_decode() {
    let linear = 0.5;
    let encoded = hlg_encode(linear);
    let decoded = hlg_decode(encoded);
    assert!((linear - decoded).abs() < 0.00001);
}

#[test]
fn test_srgb_to_xyz_red() {
    let rgb = RGB { r: 1.0, g: 0.0, b: 0.0 };
    let xyz = rgb.to_xyz_srgb();
    assert!((xyz.x - 41.2456).abs() < 0.01);
    assert!((xyz.y - 21.2673).abs() < 0.01);
    assert!((xyz.z - 1.9334).abs() < 0.01);
}

#[test]
fn test_srgb_to_xyz_white() {
    let rgb = RGB { r: 1.0, g: 1.0, b: 1.0 };
    let xyz = rgb.to_xyz_srgb();
    assert!((xyz.x - 95.047).abs() < 0.1);
    assert!((xyz.y - 100.0).abs() < 0.1);
    assert!((xyz.z - 108.883).abs() < 0.1);
}

#[test]
fn test_xyz_to_srgb_red() {
    let xyz = XYZ { x: 41.2456, y: 21.2673, z: 1.9334 };
    let rgb = xyz.to_rgb_srgb();
    assert!((rgb.r - 1.0).abs() < 0.001);
    assert!(rgb.g.abs() < 0.001);
    assert!(rgb.b.abs() < 0.001);
}

#[test]
fn test_srgb_roundtrip() {
    let original = RGB { r: 0.5, g: 0.3, b: 0.8 };
    let xyz = original.to_xyz_srgb();
    let back = xyz.to_rgb_srgb();
    assert!((original.r - back.r).abs() < 0.0001);
    assert!((original.g - back.g).abs() < 0.0001);
    assert!((original.b - back.b).abs() < 0.0001);
}
