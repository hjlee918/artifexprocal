#[cfg(test)]
mod tests {
    use crate::service::CalibrationService;
    use crate::ipc::models::CalibrationState;

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
        service.disconnect_meter("any").unwrap();
        assert_eq!(service.get_meter_info().len(), 0);
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
}
