use calibration_engine::autocal_flow::GreyscaleAutoCalFlow;
use calibration_core::state::{CalibrationTier, SessionConfig, TargetSpace, ToneCurve, WhitePoint, CalibrationState};
use calibration_storage::schema::Storage;
use calibration_storage::query::SessionQuery;
use calibration_storage::export::SessionExporter;
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
    pub wb_count: usize,
    pub model_info: String,
}

impl MockDisplay {
    fn new(model: &str) -> Self {
        Self {
            connected: false,
            lut_1d_count: 0,
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
fn history_full_flow_list_detail_export() {
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
        tier: CalibrationTier::GrayscaleOnly,
    };

    let mut flow = GreyscaleAutoCalFlow::new(config);
    let mut meter = CounterMeter::new();
    let mut display = MockDisplay::new("MockAlpha9");
    let mut pattern_gen = MockPatternGen::new();

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern_gen, &storage, &events);
    assert!(result.is_ok(), "Flow should complete: {:?}", result);
    assert!(matches!(flow.state, CalibrationState::Finished));

    // Query list_sessions
    let query = SessionQuery::new(&storage.conn);
    let (items, total) = query.list(&calibration_storage::query::SessionFilter::default(), 0, 10).unwrap();
    assert_eq!(total, 1, "Should have 1 session");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "Integration Test");
    assert_eq!(items[0].state, "finished");

    // Query get_detail
    let session_id = &items[0].id;
    let detail = query.get_detail(session_id).unwrap().expect("detail should exist");
    assert_eq!(detail.readings.len(), 5, "Should have 5 readings");
    assert!(detail.results.is_some());

    // Export CSV
    let mut csv_buf = Vec::new();
    SessionExporter::export_csv(&detail, &mut csv_buf).unwrap();
    let csv = String::from_utf8(csv_buf).unwrap();
    assert!(csv.starts_with("patch_index,target_r,target_g,target_b,measured_x,measured_y,measured_z"));
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 6); // header + 5 readings

    // Export JSON
    let mut json_buf = Vec::new();
    SessionExporter::export_json(&detail, &mut json_buf).unwrap();
    let json_str = String::from_utf8(json_buf).unwrap();
    assert!(json_str.contains(&format!("\"session_id\": \"{}\"", session_id)));
    assert!(json_str.contains("\"name\": \"Integration Test\""));
    assert!(json_str.contains("\"readings\""));
}
