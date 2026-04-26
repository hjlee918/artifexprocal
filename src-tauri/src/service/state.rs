use crate::ipc::models::{CalibrationState, DisplayInfo, MeterInfo};
use crate::service::error::CalibrationError;
use hal::traits::{DisplayController, Meter};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct CalibrationService {
    meter: Arc<Mutex<Option<Box<dyn Meter + Send>>>>,
    meter_info: Arc<Mutex<Option<MeterInfo>>>,
    display: Arc<Mutex<Option<Box<dyn DisplayController + Send>>>>,
    display_info: Arc<Mutex<Option<DisplayInfo>>>,
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
            meter_info: Arc::new(Mutex::new(None)),
            display: Arc::new(Mutex::new(None)),
            display_info: Arc::new(Mutex::new(None)),
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

        let info = MeterInfo {
            id: id.to_string(),
            name: name.to_string(),
            serial: None,
            connected: true,
            capabilities: caps,
        };
        *self.meter_info.lock() = Some(info.clone());
        Ok(info)
    }

    pub fn disconnect_meter(&self, meter_id: &str) -> Result<(), CalibrationError> {
        {
            let guard = self.meter_info.lock();
            if guard.as_ref().map_or(true, |i| i.id != meter_id) {
                return Err(CalibrationError::MeterNotFound(meter_id.to_string()));
            }
        }
        let mut guard = self.meter.lock();
        if let Some(meter) = guard.as_mut() {
            meter.disconnect();
        }
        *guard = None;
        drop(guard);
        *self.meter_info.lock() = None;
        Ok(())
    }

    pub fn get_meter_info(&self) -> Vec<MeterInfo> {
        match self.meter_info.lock().as_ref() {
            Some(info) => vec![info.clone()],
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

        let info = DisplayInfo {
            id: id.to_string(),
            name: name.to_string(),
            model: model.to_string(),
            connected: true,
            picture_mode: None,
        };
        *self.display_info.lock() = Some(info.clone());
        Ok(info)
    }

    pub fn disconnect_display(&self, display_id: &str) -> Result<(), CalibrationError> {
        {
            let guard = self.display_info.lock();
            if guard.as_ref().map_or(true, |i| i.id != display_id) {
                return Err(CalibrationError::DisplayNotFound(display_id.to_string()));
            }
        }
        let mut guard = self.display.lock();
        if let Some(display) = guard.as_mut() {
            display.disconnect();
        }
        *guard = None;
        drop(guard);
        *self.display_info.lock() = None;
        Ok(())
    }

    pub fn get_display_info(&self) -> Vec<DisplayInfo> {
        match self.display_info.lock().as_ref() {
            Some(info) => vec![info.clone()],
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
