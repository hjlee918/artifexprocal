use hal::traits::Meter;
use hal_meters::i1_pro_2::I1Pro2;
use hal_meters::spectro_trait::Spectrophotometer;

#[test]
fn test_i1_pro_2_model() {
    let meter = I1Pro2::new();
    assert_eq!(meter.model(), "i1 Pro 2");
}

#[test]
fn test_i1_pro_2_not_connected_error() {
    let mut meter = I1Pro2::new();
    let result = meter.read_xyz(500);
    assert!(result.is_err());
}

#[test]
fn test_i1_pro_2_initialize_not_connected() {
    let mut meter = I1Pro2::new();
    let result = meter.initialize();
    assert!(result.is_err());
}

#[test]
fn test_i1_pro_2_read_spectrum_not_connected() {
    let mut meter = I1Pro2::new();
    let result = meter.read_spectrum();
    assert!(result.is_err());
}
