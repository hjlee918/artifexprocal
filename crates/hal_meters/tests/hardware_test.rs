use hal::traits::Meter;
use hal_meters::i1_display_pro::I1DisplayPro;
use hal_meters::i1_pro_2::I1Pro2;
use hal_meters::spectro_trait::Spectrophotometer;

#[test]
#[ignore = "requires physical i1 Display Pro Rev.B"]
fn test_real_i1_display_pro_read() {
    let mut meter = I1DisplayPro::new();
    meter.connect().expect("Failed to connect i1 Display Pro");
    let xyz = meter.read_xyz(200).expect("Failed to read XYZ");
    println!("i1 Display Pro: X={:.3}, Y={:.3}, Z={:.3}", xyz.x, xyz.y, xyz.z);
    assert!(xyz.y > 0.0, "Luminance should be positive");
    meter.disconnect();
}

#[test]
#[ignore = "requires physical i1 Pro 2"]
fn test_real_i1_pro_2_read() {
    let mut meter = I1Pro2::new();
    meter.connect().expect("Failed to connect i1 Pro 2");
    let xyz = meter.read_xyz(500).expect("Failed to read XYZ");
    println!("i1 Pro 2: X={:.3}, Y={:.3}, Z={:.3}", xyz.x, xyz.y, xyz.z);
    assert!(xyz.y > 0.0, "Luminance should be positive");

    let spectrum = meter.read_spectrum().expect("Failed to read spectrum");
    println!("Spectrum: {:?}", &spectrum[..5]);
    meter.disconnect();
}

#[test]
#[ignore = "requires physical i1 Pro 2 with white patch"]
fn test_real_i1_pro_2_initialize() {
    let mut meter = I1Pro2::new();
    meter.connect().expect("Failed to connect i1 Pro 2");
    meter.initialize().expect("Initialization failed");
    println!("i1 Pro 2 initialized successfully");
    meter.disconnect();
}
