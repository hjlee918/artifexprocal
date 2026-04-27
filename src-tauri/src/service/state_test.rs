#[cfg(test)]
mod tests {
    use crate::service::CalibrationService;
    use crate::ipc::models::CalibrationState;
    use calibration_core::state::SessionConfig;

    #[test]
    fn test_connect_meter_known() {
        let service = CalibrationService::new();
        let info = service.connect_meter("i1-display-pro").unwrap();
        assert_eq!(info.id, "i1-display-pro");
        assert!(info.connected);
        assert!(info.capabilities.contains(&"emissive".to_string()));
    }

    #[test]
    fn test_connect_meter_unknown() {
        let service = CalibrationService::new();
        let result = service.connect_meter("fake");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_disconnect_meter() {
        let service = CalibrationService::new();
        service.connect_meter("i1-display-pro").unwrap();
        assert_eq!(service.get_meter_info().len(), 1);
        service.disconnect_meter("i1-display-pro").unwrap();
        assert_eq!(service.get_meter_info().len(), 0);
    }

    #[test]
    fn test_disconnect_meter_wrong_id() {
        let service = CalibrationService::new();
        service.connect_meter("i1-display-pro").unwrap();
        let result = service.disconnect_meter("wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_display_known() {
        let service = CalibrationService::new();
        let info = service.connect_display("lg-oled").unwrap();
        assert_eq!(info.id, "lg-oled");
        assert!(info.connected);
    }

    #[test]
    fn test_connect_display_unknown() {
        let service = CalibrationService::new();
        let result = service.connect_display("fake-display");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_disconnect_display() {
        let service = CalibrationService::new();
        service.connect_display("lg-oled").unwrap();
        assert_eq!(service.get_display_info().len(), 1);
        service.disconnect_display("lg-oled").unwrap();
        assert_eq!(service.get_display_info().len(), 0);
    }

    #[test]
    fn test_disconnect_display_wrong_id() {
        let service = CalibrationService::new();
        service.connect_display("lg-oled").unwrap();
        let result = service.disconnect_display("wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_device_inventory() {
        let service = CalibrationService::new();
        let devices = service.get_device_inventory();
        assert_eq!(devices.len(), 4);
        assert!(devices.iter().any(|d| d.id == "i1-display-pro"));
        assert!(devices.iter().any(|d| d.id == "lg-oled"));
    }

    #[test]
    fn test_state_transitions() {
        let service = CalibrationService::new();
        assert!(matches!(service.get_state(), CalibrationState::Idle));
        service.set_state(CalibrationState::Measuring);
        assert!(matches!(service.get_state(), CalibrationState::Measuring));
        service.set_state(CalibrationState::Idle);
        assert!(matches!(service.get_state(), CalibrationState::Idle));
    }

    fn test_session_config() -> SessionConfig {
        SessionConfig {
            name: "test".into(),
            target_space: calibration_core::state::TargetSpace::Bt709,
            tone_curve: calibration_core::state::ToneCurve::Gamma(2.4),
            white_point: calibration_core::state::WhitePoint::D65,
            patch_count: 21,
            reads_per_patch: 5,
            settle_time_ms: 1000,
            stability_threshold: None,
        }
    }

    #[test]
    fn test_start_calibration_session_returns_id() {
        let service = CalibrationService::new();
        let id = service.start_calibration_session(test_session_config()).unwrap();
        assert!(!id.is_empty());
        assert!(id.starts_with("cal-"));
    }

    #[test]
    fn test_second_session_returns_in_progress_error() {
        let service = CalibrationService::new();
        let _ = service.start_calibration_session(test_session_config()).unwrap();
        let result = service.start_calibration_session(test_session_config());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("already in progress"));
    }

    #[test]
    fn test_get_active_session_id() {
        let service = CalibrationService::new();
        assert_eq!(service.get_active_session_id(), None);
        let id = service.start_calibration_session(test_session_config()).unwrap();
        assert_eq!(service.get_active_session_id(), Some(id));
    }

    #[test]
    fn test_end_session_clears_active() {
        let service = CalibrationService::new();
        let _ = service.start_calibration_session(test_session_config()).unwrap();
        assert!(service.get_active_session_id().is_some());
        service.end_session();
        assert_eq!(service.get_active_session_id(), None);
    }
}
