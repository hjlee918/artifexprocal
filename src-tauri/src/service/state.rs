use crate::ipc::models::{CalibrationState, DisplayInfo, MeterInfo};
use crate::service::error::CalibrationError;
use hal::traits::{DisplayController, Meter};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct CalibrationService {
    meter: Arc<Mutex<Option<Box<dyn Meter + Send>>>>,
    display: Arc<Mutex<Option<Box<dyn DisplayController + Send>>>>,
    state: Arc<Mutex<CalibrationState>>,
}

impl Default for CalibrationService {
    fn default() -> Self {
        Self::new()
    }
}

impl CalibrationService {
    pub fn new() -> Self {
        Self {
            meter: Arc::new(Mutex::new(None)),
            display: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(CalibrationState::Idle)),
        }
    }

    pub fn get_state(&self) -> CalibrationState {
        self.state.lock().clone()
    }

    pub fn set_state(&self, state: CalibrationState) {
        *self.state.lock() = state;
    }

    pub fn connect_meter(&self, meter_id: &str) -> Result<MeterInfo, CalibrationError> {
        let known_meters: Vec<(&str, &str, Vec<String>)> = vec![
            (
                "i1-display-pro",
                "i1 Display Pro Rev.B",
                vec!["emissive".into(), "xyz".into()],
            ),
            (
                "i1-pro-2",
                "i1 Pro 2",
                vec!["emissive".into(), "xyz".into(), "spectrum".into()],
            ),
        ];

        let (id, name, caps) = known_meters
            .into_iter()
            .find(|(id, _, _)| *id == meter_id)
            .ok_or_else(|| CalibrationError::MeterNotFound(meter_id.to_string()))?;

        let mut fake = hal::mocks::FakeMeter::default();
        let _ = fake.connect();
        *self.meter.lock() = Some(Box::new(fake));

        Ok(MeterInfo {
            id: id.to_string(),
            name: name.to_string(),
            serial: None,
            connected: true,
            capabilities: caps,
        })
    }

    pub fn disconnect_meter(&self, _meter_id: &str) -> Result<(), CalibrationError> {
        if let Some(meter) = self.meter.lock().as_mut() {
            meter.disconnect();
        }
        *self.meter.lock() = None;
        Ok(())
    }

    pub fn get_meter_info(&self) -> Vec<MeterInfo> {
        let guard = self.meter.lock();
        match guard.as_ref() {
            Some(meter) => vec![MeterInfo {
                id: meter.model().to_string(),
                name: meter.model().to_string(),
                serial: None,
                connected: true,
                capabilities: vec!["emissive".into(), "xyz".into()],
            }],
            None => vec![],
        }
    }

    pub fn connect_display(&self, display_id: &str) -> Result<DisplayInfo, CalibrationError> {
        let known_displays: Vec<(&str, &str, &str)> = vec![
            ("lg-oled", "LG OLED", "LG OLED C1/C2/C3"),
            ("sony-projector", "Sony Projector", "Sony VPL-VW385ES"),
        ];

        let (id, name, model) = known_displays
            .into_iter()
            .find(|(id, _, _)| *id == display_id)
            .ok_or_else(|| CalibrationError::DisplayNotFound(display_id.to_string()))?;

        let mut fake = hal::mocks::FakeDisplayController::default();
        let _ = fake.connect();
        *self.display.lock() = Some(Box::new(fake));

        Ok(DisplayInfo {
            id: id.to_string(),
            name: name.to_string(),
            model: model.to_string(),
            connected: true,
            picture_mode: None,
        })
    }

    pub fn disconnect_display(&self, _display_id: &str) -> Result<(), CalibrationError> {
        if let Some(display) = self.display.lock().as_mut() {
            display.disconnect();
        }
        *self.display.lock() = None;
        Ok(())
    }

    pub fn get_display_info(&self) -> Vec<DisplayInfo> {
        let guard = self.display.lock();
        match guard.as_ref() {
            Some(_) => vec![DisplayInfo {
                id: "connected-display".to_string(),
                name: "Connected Display".to_string(),
                model: "Unknown".to_string(),
                connected: true,
                picture_mode: None,
            }],
            None => vec![],
        }
    }

    pub fn get_device_inventory(&self) -> Vec<crate::ipc::models::DeviceInfo> {
        vec![
            crate::ipc::models::DeviceInfo {
                id: "i1-display-pro".to_string(),
                name: "i1 Display Pro Rev.B".to_string(),
                device_type: "meter".to_string(),
                available: true,
            },
            crate::ipc::models::DeviceInfo {
                id: "i1-pro-2".to_string(),
                name: "i1 Pro 2".to_string(),
                device_type: "meter".to_string(),
                available: true,
            },
            crate::ipc::models::DeviceInfo {
                id: "lg-oled".to_string(),
                name: "LG OLED".to_string(),
                device_type: "display".to_string(),
                available: true,
            },
            crate::ipc::models::DeviceInfo {
                id: "sony-projector".to_string(),
                name: "Sony Projector".to_string(),
                device_type: "display".to_string(),
                available: true,
            },
        ]
    }
}
