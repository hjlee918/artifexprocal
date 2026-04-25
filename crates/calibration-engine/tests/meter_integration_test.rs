use calibration_engine::autocal_flow::*;
use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint};
use calibration_storage::schema::Storage;
use calibration_engine::events::EventChannel;
use hal::traits::{Meter, DisplayController, PatternGenerator};
use hal::error::{MeterError, DisplayError, PatternGenError};
use hal::types::{Lut1D, Lut3D, RGBGain};
use color_science::types::{XYZ, RGB};

struct SimulatedMeter {
    connected: bool,
    call_count: usize,
}

impl SimulatedMeter {
    fn new() -> Self {
        Self { connected: false, call_count: 0 }
    }
}

impl Meter for SimulatedMeter {
    fn connect(&mut self) -> Result<(), MeterError> {
        self.connected = true;
        Ok(())
    }
    fn disconnect(&mut self) {
        self.connected = false;
    }
    fn read_xyz(&mut self, _ms: u32) -> Result<XYZ, MeterError> {
        if !self.connected {
            return Err(MeterError::ConnectionFailed("not connected".to_string()));
        }
        self.call_count += 1;
        // Simulate D65 white
        Ok(XYZ { x: 95.047, y: 100.0, z: 108.883 })
    }
    fn model(&self) -> &str { "SimulatedMeter" }
}

struct MockDisplay;
impl DisplayController for MockDisplay {
    fn connect(&mut self) -> Result<(), DisplayError> { Ok(()) }
    fn disconnect(&mut self) {}
    fn set_picture_mode(&mut self, _m: &str) -> Result<(), DisplayError> { Ok(()) }
    fn upload_1d_lut(&mut self, _l: &Lut1D) -> Result<(), DisplayError> { Ok(()) }
    fn upload_3d_lut(&mut self, _l: &Lut3D) -> Result<(), DisplayError> { Ok(()) }
    fn set_white_balance(&mut self, _g: RGBGain) -> Result<(), DisplayError> { Ok(()) }
}

struct MockPatternGen;
impl PatternGenerator for MockPatternGen {
    fn connect(&mut self) -> Result<(), PatternGenError> { Ok(()) }
    fn disconnect(&mut self) {}
    fn display_patch(&mut self, _c: &RGB) -> Result<(), PatternGenError> { Ok(()) }
}

#[test]
fn test_calibration_with_simulated_meter() {
    let storage = Storage::new_in_memory().unwrap();
    let events = EventChannel::new(64);
    let config = SessionConfig {
        name: "MeterIntegration".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 5,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
    };

    let mut flow = GreyscaleAutoCalFlow::new(config);
    let mut meter = SimulatedMeter::new();
    let mut display = MockDisplay;
    let mut pattern = MockPatternGen;

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern, &storage, &events);
    assert!(result.is_ok());
    assert_eq!(meter.call_count, 5); // 5 patches * 1 read
}
