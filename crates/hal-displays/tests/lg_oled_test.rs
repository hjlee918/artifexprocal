use hal_displays::lg_oled::*;

#[test]
fn test_lg_oled_direct_create() {
    let ctrl = LgOledController::direct("192.168.1.100", 3000);
    assert!(ctrl.is_direct());
}

#[test]
fn test_lg_oled_devicecontrol_create() {
    let ctrl = LgOledController::devicecontrol(81);
    assert!(ctrl.is_devicecontrol());
}

#[test]
fn test_lg_oled_pairing_flow() {
    let mut ctrl = LgOledController::direct("192.168.1.100", 3000);
    assert!(ctrl.pairing_state().is_idle());
    ctrl.request_pin();
    assert!(ctrl.pairing_state().is_waiting_for_pin());
    ctrl.submit_pin("1234");
    assert!(ctrl.pairing_state().is_authenticated());
    assert_eq!(ctrl.pairing_state().client_key(), Some("key_1234"));
}
