use hal::traits::PatternGeneratorExt;
use hal_patterns::pgenerator::*;

#[test]
fn test_pgenerator_list_patterns() {
    let ctrl = PGeneratorController::devicecontrol(81);
    let patterns = ctrl.list_patterns();
    assert!(patterns.contains(&"21-Point Grayscale".to_string()));
}

#[test]
fn test_pgenerator_devicecontrol_create() {
    let ctrl = PGeneratorController::devicecontrol(81);
    // DeviceControl mode, not connected yet
    assert_eq!(ctrl.list_patterns().len(), 26);
}
