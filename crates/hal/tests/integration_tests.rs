use hal::error::*;

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

use hal::devices::sony_projector::SonyProjectorController;
use hal::traits::DisplayController;

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
use hal::traits::Meter;

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
use hal::traits::PatternGenerator;

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
