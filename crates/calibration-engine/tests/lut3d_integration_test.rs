use calibration_engine::lut3d_flow::Lut3DAutoCalFlow;
use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint, CalibrationEvent, CalibrationTier, CalibrationState};
use calibration_storage::schema::Storage;
use calibration_storage::session_store::SessionStore;
use calibration_storage::reading_store::ReadingStore;
use hal::traits::{Meter, DisplayController, PatternGenerator};
use hal::error::{MeterError, DisplayError, PatternGenError};
use hal::types::{Lut1D, Lut3D, RGBGain};
use color_science::types::{XYZ, RGB};

struct CounterMeter {
    connected: bool,
    call_count: usize,
}

impl CounterMeter {
    fn new() -> Self {
        Self { connected: false, call_count: 0 }
    }
}

impl Meter for CounterMeter {
    fn connect(&mut self) -> Result<(), MeterError> {
        self.connected = true;
        Ok(())
    }
    fn disconnect(&mut self) {
        self.connected = false;
    }
    fn read_xyz(&mut self, _integration_time_ms: u32) -> Result<XYZ, MeterError> {
        if !self.connected {
            return Err(MeterError::ConnectionFailed("not connected".to_string()));
        }
        let patch_index = self.call_count / 3;
        let level = patch_index as f64 / 20.0;
        let y = level.powf(2.2) * 100.0;
        self.call_count += 1;
        Ok(XYZ {
            x: y * 0.3127 / 0.3290,
            y,
            z: y * (1.0 - 0.3127 - 0.3290) / 0.3290,
        })
    }
    fn model(&self) -> &str {
        "CounterMeter"
    }
}

struct MockDisplay {
    connected: bool,
    pub lut_1d_count: usize,
    pub lut_3d_count: usize,
    pub wb_count: usize,
    pub model_info: String,
}

impl MockDisplay {
    fn new(model: &str) -> Self {
        Self {
            connected: false,
            lut_1d_count: 0,
            lut_3d_count: 0,
            wb_count: 0,
            model_info: model.to_string(),
        }
    }
}

impl DisplayController for MockDisplay {
    fn connect(&mut self) -> Result<(), DisplayError> {
        self.connected = true;
        Ok(())
    }
    fn disconnect(&mut self) {
        self.connected = false;
    }
    fn model(&self) -> &str { &self.model_info }
    fn set_picture_mode(&mut self, _mode: &str) -> Result<(), DisplayError> {
        Ok(())
    }
    fn upload_1d_lut(&mut self, _lut: &Lut1D) -> Result<(), DisplayError> {
        self.lut_1d_count += 1;
        Ok(())
    }
    fn upload_3d_lut(&mut self, _lut: &Lut3D) -> Result<(), DisplayError> {
        self.lut_3d_count += 1;
        Ok(())
    }
    fn set_white_balance(&mut self, _gains: RGBGain) -> Result<(), DisplayError> {
        self.wb_count += 1;
        Ok(())
    }
}

struct MockPatternGen {
    connected: bool,
    pub patch_count: usize,
}

impl MockPatternGen {
    fn new() -> Self {
        Self { connected: false, patch_count: 0 }
    }
}

impl PatternGenerator for MockPatternGen {
    fn connect(&mut self) -> Result<(), PatternGenError> {
        self.connected = true;
        Ok(())
    }
    fn disconnect(&mut self) {
        self.connected = false;
    }
    fn display_patch(&mut self, _color: &RGB) -> Result<(), PatternGenError> {
        self.patch_count += 1;
        Ok(())
    }
}

#[test]
fn lut3d_flow_grayscale_only_completes() {
    let storage = Storage::new_in_memory().unwrap();
    let events = calibration_engine::events::EventChannel::new(4096);
    let mut rx = events.subscribe();

    let config = SessionConfig {
        name: "Integration Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.4),
        white_point: WhitePoint::D65,
        patch_count: 5,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
    };

    let mut flow = Lut3DAutoCalFlow::new(config);
    let mut meter = CounterMeter::new();
    let mut display = MockDisplay::new("MockAlpha9");
    let mut pattern_gen = MockPatternGen::new();

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern_gen, &storage, &events);
    assert!(result.is_ok(), "Grayscale-only flow should complete: {:?}", result);
    assert!(matches!(flow.state, CalibrationState::Finished));
    assert!(flow.lut_1d.is_some());
    assert!(flow.lut_3d.is_none());

    // Collect events
    let mut event_list = Vec::new();
    while let Ok(evt) = rx.try_recv() {
        event_list.push(evt);
    }

    // Verify device connection events
    let devices: Vec<String> = event_list
        .iter()
        .filter_map(|e| match e {
            CalibrationEvent::DeviceConnected { device } => Some(device.clone()),
            _ => None,
        })
        .collect();
    assert!(devices.contains(&"meter".to_string()));
    assert!(devices.contains(&"display".to_string()));
    assert!(devices.contains(&"pattern_gen".to_string()));

    // Verify progress events cover all patches
    let progress_events: Vec<_> = event_list
        .iter()
        .filter_map(|e| match e {
            CalibrationEvent::ProgressUpdated { current, total } => Some((*current, *total)),
            _ => None,
        })
        .collect();
    assert_eq!(progress_events.last(), Some(&(5, 5)));

    // Verify analysis and LUT events
    assert!(event_list.iter().any(|e| matches!(e, CalibrationEvent::AnalysisComplete { .. })));
    assert!(event_list.iter().any(|e| matches!(e, CalibrationEvent::LutGenerated { .. })));
    assert!(event_list.iter().any(|e| matches!(e, CalibrationEvent::CorrectionsUploaded)));
    assert!(event_list.iter().any(|e| matches!(e, CalibrationEvent::SessionComplete { .. })));

    // Verify display received 1D LUT but not 3D LUT
    assert_eq!(display.lut_1d_count, 1);
    assert_eq!(display.lut_3d_count, 0);
    assert_eq!(display.wb_count, 1);
    assert_eq!(pattern_gen.patch_count, 5);

    // Verify session persisted with state 'finished'
    let _session_store = SessionStore::new(&storage.conn);
    let sessions: Vec<String> = storage.conn
        .prepare("SELECT id FROM sessions WHERE state = 'finished'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(sessions.len(), 1);

    // Verify readings persisted
    let reading_store = ReadingStore::new(&storage.conn);
    let count: i64 = storage.conn
        .query_row("SELECT COUNT(*) FROM readings", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 5); // 5 patches * 1 read

    let session_id = &sessions[0];
    for patch_index in 0..5 {
        let readings = reading_store.load_for_patch(session_id, patch_index, "cal").unwrap();
        assert_eq!(readings.len(), 1, "patch {} should have 1 reading", patch_index);
    }
}

#[test]
fn lut3d_flow_full3d_completes() {
    let storage = Storage::new_in_memory().unwrap();
    let events = calibration_engine::events::EventChannel::new(4096);
    let mut rx = events.subscribe();

    let config = SessionConfig {
        name: "Integration Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.4),
        white_point: WhitePoint::D65,
        patch_count: 5,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::Full3D,
    };

    let mut flow = Lut3DAutoCalFlow::new(config);
    let mut meter = CounterMeter::new();
    let mut display = MockDisplay::new("MockAlpha9");
    let mut pattern_gen = MockPatternGen::new();

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern_gen, &storage, &events);
    assert!(result.is_ok(), "Full 3D flow should complete: {:?}", result);
    assert!(matches!(flow.state, CalibrationState::Finished));
    assert!(flow.lut_1d.is_some());
    assert!(flow.lut_3d.is_some());

    // Collect events
    let mut event_list = Vec::new();
    while let Ok(evt) = rx.try_recv() {
        event_list.push(evt);
    }

    // Verify at least 2 LutGenerated events (1D and 3D)
    let lut_events: Vec<_> = event_list
        .iter()
        .filter(|e| matches!(e, CalibrationEvent::LutGenerated { .. }))
        .collect();
    assert!(lut_events.len() >= 2, "Should have LUT generated events for both 1D and 3D");

    // Verify display received both 1D and 3D LUTs
    assert_eq!(display.lut_1d_count, 1);
    assert_eq!(display.lut_3d_count, 1);
    assert_eq!(display.wb_count, 1);

    // Full3D generates more patches than grayscale-only
    assert!(pattern_gen.patch_count >= 630, "Full3D should generate many patches, got {}", pattern_gen.patch_count);

    // Verify session persisted
    let sessions: Vec<String> = storage.conn
        .prepare("SELECT id FROM sessions WHERE state = 'finished'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(sessions.len(), 1);
}

#[test]
fn lut3d_flow_full3d_alpha7_downsamples_to_17() {
    let storage = Storage::new_in_memory().unwrap();
    let events = calibration_engine::events::EventChannel::new(4096);

    let config = SessionConfig {
        name: "Integration Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.4),
        white_point: WhitePoint::D65,
        patch_count: 5,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::Full3D,
    };

    let mut flow = Lut3DAutoCalFlow::new(config);
    let mut meter = CounterMeter::new();
    let mut display = MockDisplay::new("LG Alpha 7");
    let mut pattern_gen = MockPatternGen::new();

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern_gen, &storage, &events);
    assert!(result.is_ok(), "Full 3D flow on Alpha 7 should complete: {:?}", result);
    assert!(matches!(flow.state, CalibrationState::Finished));

    // Alpha 7 should get a 17³ LUT after downsampling
    let lut_3d = flow.lut_3d.unwrap();
    assert_eq!(lut_3d.size, 17, "Alpha 7 should receive 17³ LUT");
}
