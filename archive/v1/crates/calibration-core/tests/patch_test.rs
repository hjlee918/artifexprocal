use calibration_core::patch::{Patch, GreyscalePatchSet};
use color_science::types::{RGB, XYZ};

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

#[test]
fn patch_with_measurement() {
    let patch = Patch::with_measurement(
        RGB { r: 1.0, g: 0.5, b: 0.0 },
        XYZ { x: 50.0, y: 30.0, z: 5.0 },
    );
    assert_eq!(patch.target_rgb, RGB { r: 1.0, g: 0.5, b: 0.0 });
    assert_eq!(patch.measured_xyz, Some(XYZ { x: 50.0, y: 30.0, z: 5.0 }));
}

#[test]
fn greyscale_patch_set_uses_new_constructor() {
    let set = GreyscalePatchSet::new(5);
    assert_eq!(set.len(), 5);
    assert!(set.patches[0].measured_xyz.is_none());
    assert_eq!(set.patches[4].target_rgb, RGB { r: 1.0, g: 1.0, b: 1.0 });
}
