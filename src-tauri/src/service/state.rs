use crate::ipc::models::{CalibrationState, DisplayInfo, MeterInfo};
use crate::service::error::CalibrationError;
use calibration_core::state::SessionConfig;
use calibration_storage::schema::Storage;
use color_science::types::{RGB, XYZ};
use hal::traits::{DisplayController, Meter, PatternGenerator};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::AppHandle;

struct CalibrationSession {
    session_id: String,
    config: SessionConfig,
    _pre_readings: Vec<(RGB, XYZ)>,
}

pub struct CalibrationService {
    meter: Arc<Mutex<Option<Box<dyn Meter + Send>>>>,
    meter_info: Arc<Mutex<Option<MeterInfo>>>,
    reference_meter: Arc<Mutex<Option<Box<dyn Meter + Send>>>>,
    reference_meter_info: Arc<Mutex<Option<MeterInfo>>>,
    display: Arc<Mutex<Option<Box<dyn DisplayController + Send>>>>,
    display_info: Arc<Mutex<Option<DisplayInfo>>>,
    pattern_gen: Arc<Mutex<Option<Box<dyn PatternGenerator + Send>>>>,
    state: Arc<Mutex<CalibrationState>>,
    use_mocks: bool,
    active_session: Arc<Mutex<Option<CalibrationSession>>>,
    storage: Arc<Mutex<Storage>>,
    abort_flag: Arc<AtomicBool>,
    pub(crate) active_manual_flow: Arc<Mutex<Option<calibration_engine::manual_flow::ManualCalibrationFlow>>>,
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
        let storage = Storage::new_in_memory().expect("Failed to initialize SQLite storage");
        Self {
            meter: Arc::new(Mutex::new(None)),
            meter_info: Arc::new(Mutex::new(None)),
            reference_meter: Arc::new(Mutex::new(None)),
            reference_meter_info: Arc::new(Mutex::new(None)),
            display: Arc::new(Mutex::new(None)),
            display_info: Arc::new(Mutex::new(None)),
            pattern_gen: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(CalibrationState::Idle)),
            use_mocks,
            active_session: Arc::new(Mutex::new(None)),
            storage: Arc::new(Mutex::new(storage)),
            abort_flag: Arc::new(AtomicBool::new(false)),
            active_manual_flow: Arc::new(Mutex::new(None)),
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
            _pre_readings: Vec::new(),
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
            if guard.as_ref().is_none_or(|i| i.id != meter_id) {
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

    pub fn is_meter_connected(&self, meter_id: &str) -> bool {
        match self.meter_info.lock().as_ref() {
            Some(info) => info.id == meter_id,
            None => false,
        }
    }

    pub fn connect_reference_meter(&self, meter_id: &str) -> Result<MeterInfo, CalibrationError> {
        let known_meters: Vec<(&str, &str, Vec<String>)> = vec![
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
            *self.reference_meter.lock() = Some(Box::new(fake));
        } else {
            let mut real: Box<dyn Meter + Send> = match meter_id {
                "i1-pro-2" => Box::new(hal_meters::i1_pro_2::I1Pro2::new()),
                _ => unreachable!(),
            };
            real.connect().map_err(|e| CalibrationError::Internal(e.to_string()))?;
            *self.reference_meter.lock() = Some(real);
        }

        *self.reference_meter_info.lock() = Some(info.clone());
        Ok(info)
    }

    pub fn disconnect_reference_meter(&self, meter_id: &str) -> Result<(), CalibrationError> {
        {
            let guard = self.reference_meter_info.lock();
            if guard.as_ref().is_none_or(|i| i.id != meter_id) {
                return Err(CalibrationError::MeterNotFound(meter_id.to_string()));
            }
        }
        let mut guard = self.reference_meter.lock();
        if let Some(meter) = guard.as_mut() {
            meter.disconnect();
        }
        *guard = None;
        drop(guard);
        *self.reference_meter_info.lock() = None;
        Ok(())
    }

    pub fn get_reference_meter_info(&self) -> Vec<MeterInfo> {
        match self.reference_meter_info.lock().as_ref() {
            Some(info) => vec![info.clone()],
            None => vec![],
        }
    }

    pub fn is_reference_meter_connected(&self, meter_id: &str) -> bool {
        match self.reference_meter_info.lock().as_ref() {
            Some(info) => info.id == meter_id,
            None => false,
        }
    }

    pub fn storage_conn(&self) -> parking_lot::MutexGuard<'_, calibration_storage::schema::Storage> {
        self.storage.lock()
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

    pub fn connect_pattern_generator(&self) -> Result<(), CalibrationError> {
        if self.use_mocks {
            let mut fake = hal::mocks::FakePatternGenerator::default();
            let _ = fake.connect();
            *self.pattern_gen.lock() = Some(Box::new(fake));
        } else {
            // For now, always use FakePatternGenerator until real iTPG/PGenerator is wired
            let mut fake = hal::mocks::FakePatternGenerator::default();
            let _ = fake.connect();
            *self.pattern_gen.lock() = Some(Box::new(fake));
        }
        Ok(())
    }

    pub fn disconnect_display(&self, display_id: &str) -> Result<(), CalibrationError> {
        {
            let guard = self.display_info.lock();
            if guard.as_ref().is_none_or(|i| i.id != display_id) {
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

    pub fn request_abort(&self) {
        self.abort_flag.store(true, Ordering::SeqCst);
    }

    pub fn clear_abort(&self) {
        self.abort_flag.store(false, Ordering::SeqCst);
    }

    pub fn is_aborted(&self) -> bool {
        self.abort_flag.load(Ordering::SeqCst)
    }

    pub fn run_calibration(
        &self,
        app: AppHandle,
        session_id: String,
    ) -> Result<(), CalibrationError> {
        let config = {
            let guard = self.active_session.lock();
            let session = guard.as_ref().ok_or(CalibrationError::SessionNotFound(session_id.clone()))?;
            session.config.clone()
        };

        // Connect pattern generator if not already connected
        {
            let guard = self.pattern_gen.lock();
            if guard.is_none() {
                drop(guard);
                self.connect_pattern_generator()?;
            }
        }

        // Verify all hardware is connected
        {
            let meter_guard = self.meter.lock();
            if meter_guard.is_none() {
                return Err(CalibrationError::NoHardwareConnected { device: "meter".into() });
            }
        }
        {
            let display_guard = self.display.lock();
            if display_guard.is_none() {
                return Err(CalibrationError::NoHardwareConnected { device: "display".into() });
            }
        }
        {
            let pg_guard = self.pattern_gen.lock();
            if pg_guard.is_none() {
                return Err(CalibrationError::NoHardwareConnected { device: "pattern generator".into() });
            }
        }

        self.clear_abort();
        self.set_state(CalibrationState::Connecting);

        let abort_flag = self.abort_flag.clone();
        let app_clone = app.clone();
        let meter_arc = self.meter.clone();
        let display_arc = self.display.clone();
        let pattern_gen_arc = self.pattern_gen.clone();
        let state_arc = self.state.clone();
        let active_session_arc = self.active_session.clone();

        std::thread::spawn(move || {
            let use_3d = config.tier != calibration_core::state::CalibrationTier::GrayscaleOnly;

            let storage = match Storage::new_in_memory() {
                Ok(s) => s,
                Err(e) => {
                    crate::ipc::events::emit_error_occurred(
                        &app_clone, "error".into(), format!("Storage init failed: {}", e), "run_calibration".into()
                    );
                    return;
                }
            };

            let events = calibration_engine::events::EventChannel::new(256);
            let mut rx = events.subscribe();

            // Spawn event bridge
            let bridge_app = app_clone.clone();
            let bridge_sid = session_id.clone();
            std::thread::spawn(move || {
                while let Ok(event) = rx.blocking_recv() {
                    crate::ipc::events::emit_engine_event(&bridge_app, &bridge_sid, event);
                }
            });

            // Lock hardware for the duration of the flow
            let mut meter_guard = meter_arc.lock();
            let mut display_guard = display_arc.lock();
            let mut pg_guard = pattern_gen_arc.lock();

            let meter = &mut **meter_guard.as_mut().unwrap();
            let display = &mut **display_guard.as_mut().unwrap();
            let pattern_gen = &mut **pg_guard.as_mut().unwrap();

            let result = if use_3d {
                let mut flow = calibration_engine::lut3d_flow::Lut3DAutoCalFlow::new(config);
                flow.run_sync(meter, display, pattern_gen, &storage, &events)
            } else {
                let mut flow = calibration_engine::autocal_flow::GreyscaleAutoCalFlow::new(config);
                flow.run_sync(meter, display, pattern_gen, &storage, &events)
            };

            if let Err(e) = result {
                if abort_flag.load(Ordering::SeqCst) {
                    crate::ipc::events::emit_error_occurred(
                        &app_clone, "warning".into(), "Calibration aborted".into(), "run_calibration".into()
                    );
                } else {
                    crate::ipc::events::emit_error_occurred(
                        &app_clone, "error".into(), e.to_string(), "run_calibration".into()
                    );
                }
            }

            // Disconnect hardware
            if let Some(m) = meter_guard.as_mut() { m.disconnect(); }
            if let Some(d) = display_guard.as_mut() { d.disconnect(); }
            if let Some(p) = pg_guard.as_mut() { p.disconnect(); }

            // Clear session
            *active_session_arc.lock() = None;
            *state_arc.lock() = CalibrationState::Idle;
        });

        Ok(())
    }

    pub fn run_profiling(
        &self,
        app: AppHandle,
        session_id: String,
    ) -> Result<(), CalibrationError> {
        // Verify both meters are connected
        {
            let meter_guard = self.meter.lock();
            if meter_guard.is_none() {
                return Err(CalibrationError::NoHardwareConnected { device: "meter".into() });
            }
        }
        {
            let ref_guard = self.reference_meter.lock();
            if ref_guard.is_none() {
                return Err(CalibrationError::NoHardwareConnected { device: "reference meter".into() });
            }
        }
        {
            let pg_guard = self.pattern_gen.lock();
            if pg_guard.is_none() {
                // Auto-connect pattern generator if not already connected
                drop(pg_guard);
                let _ = self.connect_pattern_generator();
            }
        }

        self.clear_abort();
        self.set_state(CalibrationState::Connecting);

        let abort_flag = self.abort_flag.clone();
        let app_clone = app.clone();
        let meter_arc = self.meter.clone();
        let reference_meter_arc = self.reference_meter.clone();
        let pattern_gen_arc = self.pattern_gen.clone();
        let state_arc = self.state.clone();

        std::thread::spawn(move || {
            let storage = match Storage::new_in_memory() {
                Ok(s) => s,
                Err(e) => {
                    crate::ipc::events::emit_error_occurred(
                        &app_clone, "error".into(), format!("Storage init failed: {}", e), "run_profiling".into()
                    );
                    return;
                }
            };

            let events = calibration_engine::events::EventChannel::new(256);
            let mut rx = events.subscribe();

            // Spawn event bridge
            let bridge_app = app_clone.clone();
            let bridge_sid = session_id.clone();
            std::thread::spawn(move || {
                while let Ok(event) = rx.blocking_recv() {
                    crate::ipc::events::emit_engine_event(&bridge_app, &bridge_sid, event);
                }
            });

            let mut meter_guard = meter_arc.lock();
            let mut ref_meter_guard = reference_meter_arc.lock();
            let mut pg_guard = pattern_gen_arc.lock();

            let meter = &mut **meter_guard.as_mut().unwrap();
            let ref_meter = &mut **ref_meter_guard.as_mut().unwrap();
            let pattern_gen = &mut **pg_guard.as_mut().unwrap();

            let config = calibration_engine::profiling_flow::ProfilingConfig::default();
            let mut flow = calibration_engine::profiling_flow::ProfilingFlow::new(config);

            let result = flow.run_sync(
                &session_id, "field_meter", "reference_meter", None,
                ref_meter, meter, pattern_gen, &storage, &events,
            );

            if let Err(e) = result {
                if abort_flag.load(Ordering::SeqCst) {
                    crate::ipc::events::emit_error_occurred(
                        &app_clone, "warning".into(), "Profiling aborted".into(), "run_profiling".into()
                    );
                } else {
                    crate::ipc::events::emit_error_occurred(
                        &app_clone, "error".into(), e.to_string(), "run_profiling".into()
                    );
                }
            }

            if let Some(m) = meter_guard.as_mut() { m.disconnect(); }
            if let Some(r) = ref_meter_guard.as_mut() { r.disconnect(); }
            if let Some(p) = pg_guard.as_mut() { p.disconnect(); }

            *state_arc.lock() = CalibrationState::Idle;
        });

        Ok(())
    }

    pub fn list_sessions(
        &self,
        filter: calibration_storage::query::SessionFilter,
        page: usize,
        per_page: usize,
    ) -> Result<(Vec<calibration_storage::query::SessionSummary>, usize), String> {
        let storage = self.storage.lock();
        let query = calibration_storage::query::SessionQuery::new(&storage.conn);
        query.list(&filter, page, per_page).map_err(|e| e.to_string())
    }

    pub fn get_session_detail(
        &self,
        session_id: &str,
    ) -> Result<Option<calibration_storage::query::SessionDetail>, String> {
        let storage = self.storage.lock();
        let query = calibration_storage::query::SessionQuery::new(&storage.conn);
        query.get_detail(session_id).map_err(|e| e.to_string())
    }

    pub fn start_manual_calibration(
        &self,
        app: AppHandle,
        config: SessionConfig,
    ) -> Result<String, CalibrationError> {
        let session_id = format!("manual-{}", uuid::Uuid::new_v4());

        {
            let guard = self.meter.lock();
            if guard.is_none() {
                return Err(CalibrationError::NoHardwareConnected { device: "meter".into() });
            }
        }
        {
            let guard = self.display.lock();
            if guard.is_none() {
                return Err(CalibrationError::NoHardwareConnected { device: "display".into() });
            }
        }
        {
            let guard = self.pattern_gen.lock();
            if guard.is_none() {
                drop(guard);
                self.connect_pattern_generator()?;
            }
        }

        let mut flow = calibration_engine::manual_flow::ManualCalibrationFlow::new(config, session_id.clone());

        let events = calibration_engine::events::EventChannel::new(64);
        let mut rx = events.subscribe();

        {
            let mut meter_guard = self.meter.lock();
            let mut display_guard = self.display.lock();
            let mut pg_guard = self.pattern_gen.lock();

            let meter = &mut **meter_guard.as_mut().unwrap();
            let display = &mut **display_guard.as_mut().unwrap();
            let pattern_gen = &mut **pg_guard.as_mut().unwrap();

            flow.start(meter, display, pattern_gen, &events)
                .map_err(|e| CalibrationError::Internal(e.to_string()))?;
        }

        drop(events);
        while let Ok(event) = rx.blocking_recv() {
            crate::ipc::events::emit_engine_event(&app, &session_id, event);
        }

        *self.active_manual_flow.lock() = Some(flow);
        Ok(session_id)
    }

    pub fn measure_manual_patch(
        &self,
        app: AppHandle,
        _session_id: String,
    ) -> Result<(usize, RGB, XYZ, f64), CalibrationError> {
        let mut flow_guard = self.active_manual_flow.lock();
        let flow = flow_guard.as_mut().ok_or(CalibrationError::SessionNotFound("No active manual calibration".into()))?;

        let events = calibration_engine::events::EventChannel::new(64);
        let mut rx = events.subscribe();

        {
            let mut meter_guard = self.meter.lock();
            let mut pg_guard = self.pattern_gen.lock();

            let meter = &mut **meter_guard.as_mut().unwrap();
            let pattern_gen = &mut **pg_guard.as_mut().unwrap();

            flow.measure_current(meter, pattern_gen, &events)
                .map_err(|e| CalibrationError::Internal(e.to_string()))?;
        }

        let session_id = flow.session_id.clone();
        drop(events);
        while let Ok(event) = rx.blocking_recv() {
            crate::ipc::events::emit_engine_event(&app, &session_id, event);
        }

        let patch = &flow.patches[flow.current_patch];
        let measured = patch.measured_xyz.ok_or(CalibrationError::Internal("No measurement".into()))?;
        let de = patch.delta_e.ok_or(CalibrationError::Internal("No dE computed".into()))?;

        Ok((flow.current_patch, patch.target_rgb, measured, de))
    }

    pub fn next_manual_patch(&self, app: AppHandle) -> Result<usize, CalibrationError> {
        let mut flow_guard = self.active_manual_flow.lock();
        let flow = flow_guard.as_mut().ok_or(CalibrationError::SessionNotFound("No active manual calibration".into()))?;

        let events = calibration_engine::events::EventChannel::new(64);
        let mut rx = events.subscribe();

        flow.next(&events).map_err(|e| CalibrationError::Internal(e.to_string()))?;

        let session_id = flow.session_id.clone();
        drop(events);
        while let Ok(event) = rx.blocking_recv() {
            crate::ipc::events::emit_engine_event(&app, &session_id, event);
        }

        Ok(flow.current_patch)
    }

    pub fn prev_manual_patch(&self, app: AppHandle) -> Result<usize, CalibrationError> {
        let mut flow_guard = self.active_manual_flow.lock();
        let flow = flow_guard.as_mut().ok_or(CalibrationError::SessionNotFound("No active manual calibration".into()))?;

        let events = calibration_engine::events::EventChannel::new(64);
        let mut rx = events.subscribe();

        flow.prev(&events).map_err(|e| CalibrationError::Internal(e.to_string()))?;

        let session_id = flow.session_id.clone();
        drop(events);
        while let Ok(event) = rx.blocking_recv() {
            crate::ipc::events::emit_engine_event(&app, &session_id, event);
        }

        Ok(flow.current_patch)
    }

    pub fn skip_manual_patch(&self, app: AppHandle) -> Result<usize, CalibrationError> {
        let mut flow_guard = self.active_manual_flow.lock();
        let flow = flow_guard.as_mut().ok_or(CalibrationError::SessionNotFound("No active manual calibration".into()))?;

        let events = calibration_engine::events::EventChannel::new(64);
        let mut rx = events.subscribe();

        flow.skip(&events).map_err(|e| CalibrationError::Internal(e.to_string()))?;

        let session_id = flow.session_id.clone();
        drop(events);
        while let Ok(event) = rx.blocking_recv() {
            crate::ipc::events::emit_engine_event(&app, &session_id, event);
        }

        Ok(flow.current_patch)
    }

    pub fn finish_manual_calibration(
        &self,
        app: AppHandle,
        apply_corrections: bool,
    ) -> Result<(), CalibrationError> {
        let mut flow_guard = self.active_manual_flow.lock();
        let flow = flow_guard.as_mut().ok_or(CalibrationError::SessionNotFound("No active manual calibration".into()))?;

        let events = calibration_engine::events::EventChannel::new(64);
        let mut rx = events.subscribe();

        {
            let mut display_guard = self.display.lock();
            let display = &mut **display_guard.as_mut().unwrap();

            flow.finish(display, &events, apply_corrections)
                .map_err(|e| CalibrationError::Internal(e.to_string()))?;
        }

        let session_id = flow.session_id.clone();
        drop(events);
        while let Ok(event) = rx.blocking_recv() {
            crate::ipc::events::emit_engine_event(&app, &session_id, event);
        }

        *self.active_manual_flow.lock() = None;
        Ok(())
    }

    pub fn abort_manual_calibration(&self) {
        *self.active_manual_flow.lock() = None;
    }

}
