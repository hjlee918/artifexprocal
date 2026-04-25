use hal_displays::calibration_commands::*;

#[test]
fn test_calibration_mode_lifecycle() {
    let mut mode = CalibrationMode::new();
    assert!(mode.is_inactive());
    mode.start("expert1");
    assert!(mode.is_active());
    assert_eq!(mode.pic_mode(), Some("expert1"));
    mode.end();
    assert!(mode.is_inactive());
}

#[test]
fn test_encode_1d_lut_size() {
    use hal::types::Lut1D;
    let lut = Lut1D {
        channels: [vec![0.0f64; 256], vec![0.0f64; 256], vec![0.0f64; 256]],
        size: 256,
    };
    let data = encode_1d_lut(&lut);
    assert_eq!(data.len(), 256 * 3 * 8);
}

#[test]
fn test_encode_white_balance() {
    use hal::types::RGBGain;
    let gains = RGBGain { r: 1.0, g: 1.0, b: 1.0 };
    let (r, g, b) = encode_white_balance(&gains);
    assert_eq!(r, 32768);
    assert_eq!(g, 32768);
    assert_eq!(b, 32768);
}

#[test]
fn test_encode_white_balance_clamped() {
    use hal::types::RGBGain;
    let gains = RGBGain { r: 3.0, g: -1.0, b: 2.0 };
    let (r, g, b) = encode_white_balance(&gains);
    assert_eq!(r, 65535); // clamped to max
    assert_eq!(g, 0);     // clamped to min
    assert_eq!(b, 65535); // clamped to max
}
