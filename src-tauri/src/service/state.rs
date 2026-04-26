use crate::ipc::models::{CalibrationState, DisplayInfo, MeterInfo};
use crate::service::error::CalibrationError;
use calibration_core::state::SessionConfig;
use color_science::types::{RGB, XYZ};
use hal::traits::{DisplayController, Meter};
use parking_lot::Mutex;
use std::sync::Arc;

struct CalibrationSession {
    session_id: String,
    config: SessionConfig,
    pre_readings: Vec<(RGB, XYZ)>,
}

pub struct CalibrationService {
    meter: Arc<Mutex<Option<Box<dyn Meter + Send>>>>,
    meter_info: Arc<Mutex<Option<MeterInfo>>>,
    display: Arc<Mutex<Option<Box<dyn DisplayController + Send>>>>,
    display_info: Arc<Mutex<Option<DisplayInfo>>>,
    state: Arc<Mutex<CalibrationState>>,
    use_mocks: bool,
    active_session: Arc<Mutex<Option<CalibrationSession>>>,
}

impl Default for CalibrationService {
    fn default() -> Self {
        Self::new()
    }
}

impl CalibrationService {
    pub fn new() -> Self {
        Self::with_mocks(true)
    }

    pub fn with_mocks(use_mocks: bool) -> Self {
        Self {
            meter: Arc::new(Mutex::new(None)),
            meter_info: Arc::new(Mutex::new(None)),
            display: Arc::new(Mutex::new(None)),
            display_info: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(CalibrationState::Idle)),
            use_mocks,
            active_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_state(&self) -> CalibrationState {
        self.state.lock().clone()
    }

    pub fn set_state(&self, state: CalibrationState) {
        *self.state.lock() = state;
    }

    pub fn start_calibration_session(
        &self,
        config: SessionConfig,
    ) -> Result<String, CalibrationError> {
        let mut guard = self.active_session.lock();
        if guard.is_some() {
            return Err(CalibrationError::SessionInProgress);
        }
        let session_id = format!("cal-{}", uuid::Uuid::new_v4());
        *guard = Some(CalibrationSession {
            session_id: session_id.clone(),
            config,
            pre_readings: Vec::new(),
        });
        Ok(session_id)
    }

    pub fn get_active_session_id(&self) -> Option<String> {
        self.active_session.lock().as_ref().map(|s| s.session_id.clone())
    }

    pub fn end_session(&self) {
        *self.active_session.lock() = None;
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

        let info = MeterInfo {
            id: id.to_string(),
            name: name.to_string(),
            serial: None,
            connected: true,
            capabilities: caps,
        };

        if self.use_mocks {
            let mut fake = hal::mocks::FakeMeter::default();
            let _ = fake.connect();
            *self.meter.lock() = Some(Box::new(fake));
        } else {
            let mut real: Box<dyn Meter + Send> = match meter_id {
                "i1-display-pro" => Box::new(hal_meters::i1_display_pro::I1DisplayPro::new()),
                "i1-pro-2" => Box::new(hal_meters::i1_pro_2::I1Pro2::new()),
                _ => unreachable!(),
            };
            real.connect().map_err(|e| CalibrationError::Internal(e.to_string()))?;
            *self.meter.lock() = Some(real);
        }

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

        let info = DisplayInfo {
            id: id.to_string(),
            name: name.to_string(),
            model: model.to_string(),
            connected: true,
            picture_mode: None,
        };

        if self.use_mocks {
            let mut fake = hal::mocks::FakeDisplayController::default();
            let _ = fake.connect();
            *self.display.lock() = Some(Box::new(fake));
        } else if display_id == "lg-oled" {
            let mut real = hal_displays::lg_oled::LgOledController::devicecontrol(3000);
            real.connect().map_err(|e| CalibrationError::Internal(e.to_string()))?;
            *self.display.lock() = Some(Box::new(real));
        } else {
            // Sony projector not yet implemented as real driver
            return Err(CalibrationError::DisplayNotFound(display_id.to_string()));
        }

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
