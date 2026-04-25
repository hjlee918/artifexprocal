use hal_displays::devicecontrol_client::*;

#[test]
fn test_lg_devicecontrol_connect_url() {
    let url = LgDeviceControlClient::connect_url("localhost", 81, "192.168.1.100", 3000);
    assert!(url.contains("lg/connect"));
    assert!(url.contains("192.168.1.100"));
    assert!(url.contains("3000"));
}

#[test]
fn test_lg_devicecontrol_start_calibration_url() {
    let url = LgDeviceControlClient::start_calibration_url("localhost", 81, "expert1");
    assert!(url.contains("lg/start_calibration"));
    assert!(url.contains("expert1"));
}

#[test]
fn test_lg_devicecontrol_create() {
    let client = LgDeviceControlClient::new("localhost", 81);
    // Just verify it constructs without panic
    let _ = client;
}
