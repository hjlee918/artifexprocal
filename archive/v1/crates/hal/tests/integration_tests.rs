use hal::error::*;
use hal::traits::*;

#[test]
fn test_meter_error_display() {
    let err = MeterError::ConnectionFailed("USB not found".to_string());
    assert_eq!(err.to_string(), "Connection failed: USB not found");
}

#[test]
fn test_meter_error_timeout() {
    let err = MeterError::ReadTimeout;
    assert_eq!(err.to_string(), "Read timeout");
}

#[test]
fn test_display_error_protocol() {
    let err = DisplayError::ProtocolError("Invalid response".to_string());
    assert_eq!(err.to_string(), "Protocol error: Invalid response");
}

#[test]
fn test_pattern_gen_error_display() {
    let err = PatternGenError::DisplayError("Patch failed".to_string());
    assert_eq!(err.to_string(), "Display error: Patch failed");
}

use hal::mocks::*;

#[test]
fn test_mock_meter_compiles() {
    use hal::traits::Meter;
    let meter = FakeMeter::default();
    let _dyn_meter: &dyn Meter = &meter;
}

#[test]
fn test_mock_display_compiles() {
    use hal::traits::DisplayController;
    let display = FakeDisplayController::default();
    let _dyn_display: &dyn DisplayController = &display;
}

#[test]
fn test_mock_pattern_gen_compiles() {
    use hal::traits::PatternGenerator;
    let gen = FakePatternGenerator::default();
    let _dyn_gen: &dyn PatternGenerator = &gen;
}

#[test]
fn test_mock_end_to_end_measurement_and_upload() {
    use color_science::types::{XYZ, RGB};
    use hal::types::{Lut1D, RGBGain};

    let mut meter = FakeMeter::with_preset(XYZ { x: 50.0, y: 75.0, z: 25.0 });
    let mut display = FakeDisplayController::default();
    let mut gen = FakePatternGenerator::default();

    meter.connect().unwrap();
    display.connect().unwrap();
    gen.connect().unwrap();

    gen.display_patch(&RGB { r: 1.0, g: 1.0, b: 1.0 }).unwrap();
    let xyz = meter.read_xyz(500).unwrap();
    assert_eq!(xyz.x, 50.0);

    let lut = Lut1D {
        channels: [vec![0.0, 1.0], vec![0.0, 1.0], vec![0.0, 1.0]],
        size: 2,
    };
    display.upload_1d_lut(&lut).unwrap();
    display.set_white_balance(RGBGain { r: 1.02, g: 1.0, b: 0.98 }).unwrap();

    assert_eq!(display.uploaded_1d_luts.len(), 1);
    assert_eq!(display.white_balance_calls.len(), 1);
    assert_eq!(display.white_balance_calls[0].r, 1.02);
    assert_eq!(gen.patch_history.len(), 1);
    assert_eq!(gen.patch_history[0].r, 1.0);
}

use hal::devices::sony_projector::SonyProjectorController;

#[test]
fn test_sony_projector_connect_valid_ip() {
    let mut display = SonyProjectorController::new("192.168.1.50");
    assert!(display.connect().is_ok());
}

#[test]
fn test_sony_projector_connect_invalid_ip() {
    let mut display = SonyProjectorController::new("bad-ip");
    assert!(display.connect().is_err());
}

use hal::devices::lg_oled::LgOledController;

#[test]
fn test_lg_oled_connect_valid_ip() {
    let mut display = LgOledController::new("192.168.1.100");
    assert!(display.connect().is_ok());
}

#[test]
fn test_lg_oled_connect_invalid_ip() {
    let mut display = LgOledController::new("not-an-ip");
    assert!(display.connect().is_err());
}

#[test]
fn test_lg_oled_set_picture_mode_stub() {
    let mut display = LgOledController::new("192.168.1.100");
    display.connect().unwrap();
    assert!(display.set_picture_mode("Cinema").is_ok());
}

use hal::devices::xrite_i1_display_pro::I1DisplayPro;

#[test]
fn test_i1_display_pro_connect() {
    let mut meter = I1DisplayPro::new("/dev/hidraw0");
    assert!(meter.connect().is_ok());
    assert_eq!(meter.model(), "i1 Display Pro Rev.B");
}

#[test]
fn test_i1_display_pro_read_xyz_stub() {
    let mut meter = I1DisplayPro::new("/dev/hidraw0");
    meter.connect().unwrap();
    let xyz = meter.read_xyz(500).unwrap();
    assert_eq!(xyz.x, 95.047);
    assert_eq!(xyz.y, 100.0);
    assert_eq!(xyz.z, 108.883);
}

use hal::devices::pgenerator::PGenerator;

#[test]
fn test_pgenerator_connect_valid_ip() {
    let mut gen = PGenerator::new("192.168.1.10");
    assert!(gen.connect().is_ok());
}

#[test]
fn test_pgenerator_connect_invalid_ip() {
    let mut gen = PGenerator::new("invalid");
    assert!(gen.connect().is_err());
}

#[test]
fn test_pgenerator_display_patch_stub() {
    let mut gen = PGenerator::new("192.168.1.10");
    gen.connect().unwrap();
    let color = color_science::types::RGB { r: 1.0, g: 0.5, b: 0.0 };
    assert!(gen.display_patch(&color).is_ok());
}

use hal::devices::xrite_i1_pro_2::I1Pro2;

#[test]
fn test_i1_pro_2_connect() {
    let mut meter = I1Pro2::new("/dev/ttyUSB0");
    assert!(meter.connect().is_ok());
    assert_eq!(meter.model(), "i1 Pro 2");
}

use hal::devices::lg_internal::LgInternalPatternGenerator;

#[test]
fn test_lg_internal_connect_valid_ip() {
    let mut gen = LgInternalPatternGenerator::new("192.168.1.100");
    assert!(gen.connect().is_ok());
}

#[test]
fn test_lg_internal_display_patch_stub() {
    let mut gen = LgInternalPatternGenerator::new("192.168.1.100");
    gen.connect().unwrap();
    let color = color_science::types::RGB { r: 0.0, g: 0.0, b: 0.0 };
    assert!(gen.display_patch(&color).is_ok());
}

use hal::types::*;
use color_science::types::RGB;

#[test]
fn test_lut1d_creation() {
    let lut = Lut1D {
        channels: [vec![0.0, 0.5, 1.0], vec![0.0, 0.5, 1.0], vec![0.0, 0.5, 1.0]],
        size: 3,
    };
    assert_eq!(lut.size, 3);
    assert_eq!(lut.channels[0][1], 0.5);
}

#[test]
fn test_lut3d_creation() {
    let lut = Lut3D {
        data: vec![RGB { r: 1.0, g: 0.0, b: 0.0 }],
        size: 1,
    };
    assert_eq!(lut.size, 1);
    assert_eq!(lut.data[0].r, 1.0);
}

#[test]
fn test_rgb_gain_creation() {
    let gain = RGBGain { r: 1.02, g: 1.0, b: 0.98 };
    assert_eq!(gain.r, 1.02);
    assert_eq!(gain.g, 1.0);
    assert_eq!(gain.b, 0.98);
}

#[test]
fn test_picture_mode_enum() {
    let mode = PictureMode::Cinema;
    assert!(matches!(mode, PictureMode::Cinema));
    let custom = PictureMode::Custom("ISF Day".to_string());
    assert!(matches!(custom, PictureMode::Custom(_)));
}
