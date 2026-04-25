use hal::traits::Meter;
use hal_meters::i1_display_pro::I1DisplayPro;
use hal_meters::i1_pro_2::I1Pro2;
use hal_meters::spectro_trait::Spectrophotometer;
use hal_meters::hid_util::HidContext;
use hal_meters::commands::{CMD_SET_EMISSIVE, CMD_GET_FIRMWARE};
use hal_meters::hid_util::{send_command, read_response, I1_DISPLAY_PRO};
use hal_meters::commands::XriteStatus;

#[test]
#[ignore = "requires physical i1 Display Pro Rev.B"]
fn test_real_i1_display_pro_read() {
    let ctx = HidContext::new().expect("Failed to init HID");
    let found = ctx.enumerate_xrite();
    println!("Found X-Rite devices: {}", found.len());
    for (info, dev) in &found {
        println!("  {} - serial={:?}", dev.name, info.serial_number());
    }

    let mut meter = I1DisplayPro::new();
    meter.connect().expect("Failed to connect i1 Display Pro");
    println!("Connected to i1 Display Pro");

    let xyz = meter.read_xyz(200).expect("Failed to read XYZ");
    println!("i1 Display Pro: X={:.3}, Y={:.3}, Z={:.3}", xyz.x, xyz.y, xyz.z);
    assert!(xyz.y > 0.0, "Luminance should be positive");
    meter.disconnect();
}

#[test]
#[ignore = "requires physical i1 Pro 2"]
fn test_real_i1_pro_2_read() {
    let ctx = HidContext::new().expect("Failed to init HID");
    let found = ctx.enumerate_xrite();
    println!("Found X-Rite devices: {}", found.len());
    for (info, dev) in &found {
        println!("  {} - serial={:?}", dev.name, info.serial_number());
    }

    let mut meter = I1Pro2::new();
    meter.connect().expect("Failed to connect i1 Pro 2");
    println!("Connected to i1 Pro 2");

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

#[test]
#[ignore = "debug raw HID communication"]
fn test_debug_i1_display_pro_raw() {
    let ctx = HidContext::new().expect("Failed to init HID");
    let mut device = ctx.open_device(&I1_DISPLAY_PRO).expect("Failed to open");

    // Get firmware
    send_command(&mut device, CMD_GET_FIRMWARE, &[]).unwrap();
    let resp = read_response(&mut device, 2000).unwrap();
    println!("Firmware response: {:02X?} (len={})", &resp, resp.len());
    if resp.len() >= 2 {
        println!("  Byte 0: {:02X} status={:?}", resp[0], XriteStatus::from_byte(resp[0]));
        println!("  Byte 1: {:02X} command echo", resp[1]);
        if resp.len() > 2 {
            let text = String::from_utf8_lossy(&resp[2..]);
            println!("  Text: {:?}", text.trim_matches('\0'));
        }
    }

    // Set emissive
    send_command(&mut device, CMD_SET_EMISSIVE, &[]).unwrap();
    let resp = read_response(&mut device, 2000).unwrap();
    println!("Emissive response: {:02X?} (len={})", &resp, resp.len());
    if resp.len() >= 2 {
        println!("  Byte 0: {:02X} status={:?}", resp[0], XriteStatus::from_byte(resp[0]));
        println!("  Byte 1: {:02X} command echo", resp[1]);
        if resp.len() > 2 {
            let text = String::from_utf8_lossy(&resp[2..]);
            println!("  Text: {:?}", text.trim_matches('\0'));
        }
    }
}

#[test]
#[ignore = "debug all HID devices"]
fn test_debug_all_hid_devices() {
    let ctx = HidContext::new().expect("Failed to init HID");
    let api = hidapi::HidApi::new().unwrap();
    println!("All HID devices:");
    for dev in api.device_list() {
        println!(
            "  VID={:04X} PID={:04X} serial={:?} prod={:?} manuf={:?}",
            dev.vendor_id(),
            dev.product_id(),
            dev.serial_number(),
            dev.product_string(),
            dev.manufacturer_string()
        );
    }
}
