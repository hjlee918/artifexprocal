use hal_meters::spectro_trait::Spectrophotometer;
use hal_meters::i1_pro_2::I1Pro2;

#[test]
fn test_wavelengths_count() {
    assert_eq!(<I1Pro2 as Spectrophotometer>::wavelengths().len(), 36);
}

#[test]
fn test_first_and_last_wavelength() {
    let waves = <I1Pro2 as Spectrophotometer>::wavelengths();
    assert_eq!(waves[0], 380.0);
    assert_eq!(waves[35], 730.0);
}
