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
