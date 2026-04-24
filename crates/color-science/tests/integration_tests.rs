use color_science::types::*;

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
