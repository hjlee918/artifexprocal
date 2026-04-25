use hal_meters::profiling::*;
use color_science::types::XYZ;

#[test]
fn test_correction_matrix_identity() {
    let identity = CorrectionMatrix::identity();
    let xyz = XYZ { x: 50.0, y: 100.0, z: 25.0 };
    let corrected = identity.apply(&xyz);
    assert!((corrected.x - 50.0).abs() < 1e-6);
    assert!((corrected.y - 100.0).abs() < 1e-6);
    assert!((corrected.z - 25.0).abs() < 1e-6);
}
