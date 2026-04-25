use calibration_engine::autocal_flow::*;
use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint};

#[test]
fn test_autocal_flow_create_and_advance() {
    let config = SessionConfig {
        name: "Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 0,
        stability_threshold: None,
    };

    let mut flow = GreyscaleAutoCalFlow::new(config);
    assert!(matches!(flow.state(), calibration_core::state::CalibrationState::Idle));

    flow.start().unwrap();
    assert!(matches!(flow.state(), calibration_core::state::CalibrationState::Connecting));
}
