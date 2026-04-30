use color_science::types::{RGB, XYZ};

fn main() {
    let rgb = RGB { r: 1.0, g: 0.0, b: 0.0 };
    let xyz = rgb.to_xyz_srgb();
    println!("sRGB red -> XYZ: x={:.6}, y={:.6}, z={:.6}", xyz.x, xyz.y, xyz.z);
    println!("  expected:        x=41.2456, y=21.2673, z=1.9334");

    let rgb = RGB { r: 1.0, g: 1.0, b: 1.0 };
    let xyz = rgb.to_xyz_srgb();
    println!("sRGB white -> XYZ: x={:.6}, y={:.6}, z={:.6}", xyz.x, xyz.y, xyz.z);
    println!("  expected:         x=95.047, y=100.0, z=108.883");

    let xyz = XYZ { x: 41.2456, y: 21.2673, z: 1.9334 };
    let rgb = xyz.to_rgb_srgb();
    println!("XYZ red -> sRGB: r={:.6}, g={:.6}, b={:.6}", rgb.r, rgb.g, rgb.b);
    println!("  expected:       r=1.0, g=0.0, b=0.0");

    let original = RGB { r: 0.5, g: 0.3, b: 0.8 };
    let xyz = original.to_xyz_srgb();
    let back = xyz.to_rgb_srgb();
    println!("sRGB roundtrip: original r={:.6}, g={:.6}, b={:.6}", original.r, original.g, original.b);
    println!("                back     r={:.6}, g={:.6}, b={:.6}", back.r, back.g, back.b);
}
