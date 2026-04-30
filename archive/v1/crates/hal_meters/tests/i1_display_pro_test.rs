use hal::traits::Meter;
use hal_meters::i1_display_pro::I1DisplayPro;

#[test]
fn test_i1_display_pro_model() {
    let meter = I1DisplayPro::new();
    assert_eq!(meter.model(), "i1 Display Pro Rev.B");
}

#[test]
fn test_i1_display_pro_default_integration_time() {
    let meter = I1DisplayPro::new();
    assert_eq!(meter.integration_time_ms(), 200);
}

#[test]
fn test_i1_display_pro_set_integration_time() {
    let mut meter = I1DisplayPro::new();
    meter.set_integration_time(500);
    assert_eq!(meter.integration_time_ms(), 500);
}

#[test]
fn test_i1_display_pro_integration_time_clamped() {
    let mut meter = I1DisplayPro::new();
    meter.set_integration_time(10);
    assert_eq!(meter.integration_time_ms(), 80);
    meter.set_integration_time(10000);
    assert_eq!(meter.integration_time_ms(), 5000);
}

#[test]
fn test_i1_display_pro_not_connected_error() {
    let mut meter = I1DisplayPro::new();
    let result = meter.read_xyz(200);
    assert!(result.is_err());
}
