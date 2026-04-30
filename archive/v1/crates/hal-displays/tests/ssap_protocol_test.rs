use hal_displays::ssap_protocol::*;

#[test]
fn test_ssap_start_calibration() {
    let msg = SsapMessage::start_calibration("expert1");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("startCalibration"));
    assert!(json.contains("expert1"));
}

#[test]
fn test_ssap_end_calibration() {
    let msg = SsapMessage::end_calibration("expert1");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("endCalibration"));
    assert!(json.contains("expert1"));
}

#[test]
fn test_ssap_upload_1d_lut() {
    let data = vec![0u8; 1024];
    let msg = SsapMessage::upload_1d_lut("expert1", &data);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("setExternalPqData"));
    assert!(json.contains("picMode"));
    assert!(json.contains("data"));
}

#[test]
fn test_ssap_upload_3d_lut() {
    let data = vec![0u8; 1024];
    let msg = SsapMessage::upload_3d_lut("expert1", "bt709", &data);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("colorSpace"));
    assert!(json.contains("bt709"));
}

#[test]
fn test_ssap_set_white_balance() {
    let msg = SsapMessage::set_white_balance(128, 129, 130);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("setWhiteBalance"));
    assert!(json.contains("128"));
    assert!(json.contains("129"));
    assert!(json.contains("130"));
}

#[test]
fn test_ssap_response_success() {
    let resp: SsapResponse = serde_json::from_str(r#"{"type":"response","id":"1"}"#).unwrap();
    assert!(resp.is_success());
}

#[test]
fn test_ssap_response_error() {
    let resp: SsapResponse = serde_json::from_str(r#"{"type":"response","id":"1","error":"fail"}"#).unwrap();
    assert!(!resp.is_success());
}
