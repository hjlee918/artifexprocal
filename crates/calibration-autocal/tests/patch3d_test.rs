use calibration_autocal::patch3d::OptimizedPatchSetGenerator;
use calibration_core::patch::PatchStrategy;

#[test]
fn optimized_subset_grayscale_only() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::Grayscale(21));
    assert_eq!(set.len(), 21);
    // All grayscale
    for patch in &set.patches {
        assert!(patch.measured_xyz.is_none());
        assert!((patch.target_rgb.r - patch.target_rgb.g).abs() < 0.001);
        assert!((patch.target_rgb.g - patch.target_rgb.b).abs() < 0.001);
    }
}

#[test]
fn optimized_subset_full3d_has_enough_patches() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::OptimizedSubset {
        grayscale_count: 33,
        color_count: 600,
    });
    assert!(set.len() >= 633, "Expected at least 633 patches, got {}", set.len());
    assert!(set.len() <= 640, "Expected at most 640 patches, got {}", set.len());
}

#[test]
fn optimized_subset_grayscale_plus_3d() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::OptimizedSubset {
        grayscale_count: 21,
        color_count: 180,
    });
    assert!(set.len() >= 200, "Expected at least 200 patches, got {}", set.len());
    // First 21 should be grayscale
    for i in 0..21 {
        let p = &set.patches[i];
        assert!((p.target_rgb.r - p.target_rgb.g).abs() < 0.001);
        assert!((p.target_rgb.g - p.target_rgb.b).abs() < 0.001);
    }
}

#[test]
fn optimized_subset_includes_corners() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::OptimizedSubset {
        grayscale_count: 21,
        color_count: 180,
    });
    let has_black = set.patches.iter().any(|p| p.target_rgb.r < 0.01 && p.target_rgb.g < 0.01 && p.target_rgb.b < 0.01);
    let has_white = set.patches.iter().any(|p| p.target_rgb.r > 0.99 && p.target_rgb.g > 0.99 && p.target_rgb.b > 0.99);
    assert!(has_black, "Should include black patch");
    assert!(has_white, "Should include white patch");
}
