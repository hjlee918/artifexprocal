use hal_meters::hid_util::*;

#[test]
fn test_vid_pid_constants() {
    assert_eq!(I1_DISPLAY_PRO.vid, 0x0765);
    assert_eq!(I1_DISPLAY_PRO.pid, 0x5020);
}

#[test]
fn test_xrite_device_name() {
    assert_eq!(I1_DISPLAY_PRO.name, "i1 Display Pro Rev.B");
}
