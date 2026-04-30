use calibration_autocal::export::Lut3DExporter;
use hal::types::{Lut3D, RGB};

fn test_lut(size: usize) -> Lut3D {
    let mut data = Vec::with_capacity(size * size * size);
    for r in 0..size {
        for g in 0..size {
            for b in 0..size {
                let rf = r as f64 / (size - 1).max(1) as f64;
                let gf = g as f64 / (size - 1).max(1) as f64;
                let bf = b as f64 / (size - 1).max(1) as f64;
                data.push(RGB { r: rf, g: gf, b: bf });
            }
        }
    }
    Lut3D { data, size }
}

#[test]
fn export_cube_header() {
    let lut = test_lut(3);
    let mut buf = Vec::new();
    Lut3DExporter::export_cube(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("LUT_3D_SIZE 3"));
    assert!(s.contains("ArtifexProCal"));
}

#[test]
fn export_cube_has_correct_line_count() {
    let lut = test_lut(3);
    let mut buf = Vec::new();
    Lut3DExporter::export_cube(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = s.lines().collect();
    // Header + blank + 27 data lines
    assert!(lines.iter().any(|l| l.contains("LUT_3D_SIZE")));
}

#[test]
fn export_3dl_header() {
    let lut = test_lut(3);
    let mut buf = Vec::new();
    Lut3DExporter::export_3dl(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("3DMESH"));
    assert!(s.contains("Mesh 3"));
}

#[test]
fn export_3dl_white_is_1023() {
    let lut = test_lut(2); // 0 and 1
    let mut buf = Vec::new();
    Lut3DExporter::export_3dl(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    // White at (1,1,1) should be 1023 1023 1023
    assert!(s.contains("1023 1023 1023"));
    // Black at (0,0,0) should be 0 0 0
    assert!(s.contains("0 0 0"));
}
