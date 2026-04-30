use calibration_core::state::*;

#[test]
fn test_session_config_creation() {
    let config = SessionConfig {
        name: "Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 500,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
            manual_patches: None,
    };
    assert_eq!(config.patch_count, 21);
    assert_eq!(config.reads_per_patch, 3);
}

#[test]
fn test_calibration_state_transitions() {
    let state = CalibrationState::Idle;
    assert!(matches!(state, CalibrationState::Idle));
}

#[test]
fn test_calibration_event_variants() {
    let event = CalibrationEvent::ProgressUpdated { current: 5, total: 21 };
    assert!(matches!(event, CalibrationEvent::ProgressUpdated { current: 5, total: 21 }));
}

#[test]
fn test_calibration_error_display() {
    let err = CalibrationError::MeterRead("Timeout".to_string());
    assert_eq!(err.to_string(), "Meter read failed: Timeout");
}
