use calibration_engine::lut3d_flow::Lut3DAutoCalFlow;
use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint, CalibrationTier};

fn test_config(tier: CalibrationTier) -> SessionConfig {
    SessionConfig {
        name: "Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.4),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 10,
        stability_threshold: None,
        tier,
        manual_patches: None,
    }
}

#[test]
fn lut3d_flow_new_is_idle() {
    let flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::GrayscaleOnly));
    assert!(matches!(flow.state, calibration_core::state::CalibrationState::Idle));
}

#[test]
fn lut3d_flow_generate_patches_grayscale_only() {
    let mut flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::GrayscaleOnly));
    flow.generate_patches();
    assert_eq!(flow.patches.as_ref().unwrap().len(), 21);
}

#[test]
fn lut3d_flow_generate_patches_grayscale_plus_3d() {
    let mut flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::GrayscalePlus3D));
    flow.generate_patches();
    assert!(flow.patches.as_ref().unwrap().len() >= 200);
}

#[test]
fn lut3d_flow_generate_patches_full_3d() {
    let mut flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::Full3D));
    flow.generate_patches();
    assert!(flow.patches.as_ref().unwrap().len() >= 630);
}
