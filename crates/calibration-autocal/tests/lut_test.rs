use calibration_autocal::lut::*;
use hal::types::Lut1D;

#[test]
fn test_lut_from_corrections_identity() {
    // Identity corrections (factor = 1.0 everywhere)
    let corrections: [Vec<f64>; 3] = [
        vec![1.0; 21],
        vec![1.0; 21],
        vec![1.0; 21],
    ];

    let lut = Lut1DGenerator::from_corrections(&corrections, 256);
    assert_eq!(lut.size, 256);

    // For identity, input 0.5 should map to ~0.5
    let idx = 128;
    assert!((lut.channels[0][idx] - 0.5).abs() < 0.02);
    assert!((lut.channels[1][idx] - 0.5).abs() < 0.02);
    assert!((lut.channels[2][idx] - 0.5).abs() < 0.02);
}
