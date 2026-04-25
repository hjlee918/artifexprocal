use hal_displays::pairing::*;

#[test]
fn test_pairing_state_transitions() {
    let mut state = PairingState::new();
    assert!(state.is_idle());
    state.request_pin();
    assert!(state.is_waiting_for_pin());
    state.submit_pin("1234");
    assert!(state.is_authenticated());
    assert_eq!(state.client_key(), Some("key_1234"));
}

#[test]
fn test_pairing_request_message() {
    let msg = PairingMessage::request_key("ArtifexProCal");
    assert!(msg.contains("getKey"));
    assert!(msg.contains("ArtifexProCal"));
}

#[test]
fn test_pairing_send_pin_message() {
    let msg = PairingMessage::send_pin("5678");
    assert!(msg.contains("sendKey"));
    assert!(msg.contains("5678"));
}

#[test]
fn test_pairing_failed_state() {
    let mut state = PairingState::new();
    state.fail("timeout".to_string());
    assert!(!state.is_idle());
    assert!(!state.is_waiting_for_pin());
    assert!(!state.is_authenticated());
    assert_eq!(state.client_key(), None);
}
