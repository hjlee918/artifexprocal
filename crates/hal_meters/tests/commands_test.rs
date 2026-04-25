use hal_meters::commands::*;

#[test]
fn test_command_codes() {
    assert_eq!(CMD_GET_FIRMWARE, 0x01);
    assert_eq!(CMD_SET_EMISSIVE, 0x02);
    assert_eq!(CMD_TRIGGER_MEASURE, 0x03);
    assert_eq!(CMD_READ_XYZ, 0x04);
    assert_eq!(CMD_READ_SPECTRUM, 0x05);
    assert_eq!(CMD_INITIALIZE, 0x06);
    assert_eq!(CMD_SET_INTEGRATION_TIME, 0x07);
}

#[test]
fn test_status_codes() {
    assert_eq!(XriteStatus::Ok as u8, 0x00);
    assert_eq!(XriteStatus::Error as u8, 0xFF);
    assert_eq!(XriteStatus::InitializationRequired as u8, 0xFE);
}

#[test]
fn test_status_from_byte() {
    assert!(XriteStatus::from_byte(0x00).is_ok());
    assert!(!XriteStatus::from_byte(0xFF).is_ok());
    assert!(!XriteStatus::from_byte(0xFE).is_ok());
    assert!(!XriteStatus::from_byte(0x01).is_ok());
}
