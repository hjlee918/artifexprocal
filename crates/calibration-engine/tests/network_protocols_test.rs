use calibration_engine::autocal_flow::*;
use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint, CalibrationTier};
use calibration_storage::schema::Storage;
use calibration_engine::events::EventChannel;
use hal::traits::{Meter, DisplayController, PatternGenerator};
use hal::error::{MeterError, DisplayError, PatternGenError};
use hal::types::{Lut1D, Lut3D, RGBGain};
use color_science::types::{XYZ, RGB};

struct MockMeter;
impl Meter for MockMeter {
    fn connect(&mut self) -> Result<(), MeterError> { Ok(()) }
    fn disconnect(&mut self) {}
    fn read_xyz(&mut self, _ms: u32) -> Result<XYZ, MeterError> {
        Ok(XYZ { x: 50.0, y: 50.0, z: 50.0 })
    }
    fn model(&self) -> &str { "MockMeter" }
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
fn test_calibration_engine_with_mock_network_devices() {
    let storage = Storage::new_in_memory().unwrap();
    let events = EventChannel::new(64);
    let config = SessionConfig {
        name: "NetworkTest".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 11,
        reads_per_patch: 2,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
    };

    let mut flow = GreyscaleAutoCalFlow::new(config);
    let mut meter = MockMeter;
    let mut display = MockDisplay;
    let mut pattern = MockPatternGen;

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern, &storage, &events
    );
    assert!(result.is_ok());
}
