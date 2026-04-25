use hal_displays::itpg::*;

#[test]
fn test_itpg_patch_color_10bit() {
    let msg = ItpgMessage::set_patch_color(512, 512, 512);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("512"));
    assert!(json.contains("displayPattern"));
}

#[test]
fn test_itpg_enable() {
    let msg = ItpgMessage::enable(true);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("true"));
    assert!(json.contains("displayPattern"));
}

#[test]
fn test_itpg_disable() {
    let msg = ItpgMessage::enable(false);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("false"));
}

#[test]
fn test_itpg_set_window() {
    let msg = ItpgMessage::set_window(100, 100, 50, 50);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("windowH"));
    assert!(json.contains("100"));
    assert!(json.contains("patchV"));
    assert!(json.contains("50"));
}

#[test]
fn test_to_10bit() {
    assert_eq!(to_10bit(0.0), 0);
    assert_eq!(to_10bit(1.0), 1023);
    assert_eq!(to_10bit(0.5), 512);
}

#[test]
fn test_to_10bit_clamped() {
    assert_eq!(to_10bit(-0.5), 0);
    assert_eq!(to_10bit(1.5), 1023);
}
