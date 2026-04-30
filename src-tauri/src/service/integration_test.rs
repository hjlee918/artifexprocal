#[cfg(test)]
mod tests {
    use crate::service::CalibrationService;

    #[test]
    fn test_full_calibration_with_mocks() {
        let service = CalibrationService::with_mocks(true);

        // Connect meter and display
        service.connect_meter("i1-display-pro").unwrap();
        service.connect_display("lg-oled").unwrap();

        // Start session
        let config = calibration_core::state::SessionConfig {
            name: "test".into(),
            target_space: calibration_core::state::TargetSpace::Bt709,
            tone_curve: calibration_core::state::ToneCurve::Gamma(2.4),
            white_point: calibration_core::state::WhitePoint::D65,
            patch_count: 5,
            reads_per_patch: 3,
            settle_time_ms: 10,
            stability_threshold: None,
            tier: calibration_core::state::CalibrationTier::GrayscaleOnly,
        };
        let session_id = service.start_calibration_session(config).unwrap();

        // Verify session is active
        assert_eq!(service.get_active_session_id(), Some(session_id));

        // End session
        service.end_session();
        assert_eq!(service.get_active_session_id(), None);
    }
}
