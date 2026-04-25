use calibration_core::patch::*;
use color_science::types::RGB;

#[test]
fn test_greyscale_patch_set_count() {
    let patches = GreyscalePatchSet::new(21);
    assert_eq!(patches.len(), 21);
}

#[test]
fn test_greyscale_patch_set_first_and_last() {
    let patches = GreyscalePatchSet::new(21);
    let first = patches.get(0);
    let last = patches.get(20);

    assert_eq!(first.target_rgb, RGB { r: 0.0, g: 0.0, b: 0.0 });
    assert_eq!(last.target_rgb, RGB { r: 1.0, g: 1.0, b: 1.0 });
}

#[test]
fn test_greyscale_patch_set_monotonic() {
    let patches = GreyscalePatchSet::new(21);
    for i in 1..patches.len() {
        let prev = patches.get(i - 1).target_rgb.r;
        let curr = patches.get(i).target_rgb.r;
        assert!(curr > prev, "Greyscale patches should be monotonically increasing");
    }
}
