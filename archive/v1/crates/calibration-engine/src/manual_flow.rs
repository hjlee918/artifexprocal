use calibration_core::state::{CalibrationEvent, CalibrationError, SessionConfig, TargetSpace, WhitePoint};
use calibration_core::measure::MeasurementLoop;
use color_science::delta_e::delta_e_2000;
use color_science::types::{RGB, XYZ};
use hal::traits::{Meter, DisplayController, PatternGenerator};
use crate::events::EventChannel;
use std::time::Duration;
use std::thread;

/// A single patch in a manual calibration session, with optional measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPatch {
    pub target_rgb: RGB,
    pub measured_xyz: Option<XYZ>,
    pub delta_e: Option<f64>,
    pub skipped: bool,
    pub patch_type: String,
}

impl ManualPatch {
    pub fn new(target_rgb: RGB, patch_type: &str) -> Self {
        Self {
            target_rgb,
            measured_xyz: None,
            delta_e: None,
            skipped: false,
            patch_type: patch_type.to_string(),
        }
    }
}

/// State of a manual calibration session.
#[derive(Debug, Clone, PartialEq)]
pub enum ManualCalibrationState {
    Idle,
    Connecting,
    Connected,
    Ready,
    Measuring,
    Measured,
    GeneratingLut,
    Finished,
    Error(String),
}

/// Convert linear RGB to XYZ for a given target space.
fn rgb_to_target_xyz(rgb: &RGB, target_space: &TargetSpace) -> XYZ {
    match target_space {
        TargetSpace::Bt709 => rgb.to_xyz_srgb(),
        TargetSpace::Bt2020 => {
            const M: [[f64; 3]; 3] = [
                [63.5538, 14.8905, 16.8879],
                [26.1579, 67.8893, 5.9528],
                [0.0000, 2.8154, 106.0723],
            ];
            XYZ {
                x: M[0][0] * rgb.r + M[0][1] * rgb.g + M[0][2] * rgb.b,
                y: M[1][0] * rgb.r + M[1][1] * rgb.g + M[1][2] * rgb.b,
                z: M[2][0] * rgb.r + M[2][1] * rgb.g + M[2][2] * rgb.b,
            }
        }
        TargetSpace::DciP3 => {
            const M: [[f64; 3]; 3] = [
                [48.052, 26.474, 19.643],
                [22.718, 69.078, 8.204],
                [0.0, 4.478, 104.035],
            ];
            XYZ {
                x: M[0][0] * rgb.r + M[0][1] * rgb.g + M[0][2] * rgb.b,
                y: M[1][0] * rgb.r + M[1][1] * rgb.g + M[1][2] * rgb.b,
                z: M[2][0] * rgb.r + M[2][1] * rgb.g + M[2][2] * rgb.b,
            }
        }
        TargetSpace::Custom { red, green, blue, white } => {
            // Treat RGB fields as chromaticity coordinates (r=x, g=y)
            let p = [
                [red.r / red.g, green.r / green.g, blue.r / blue.g],
                [1.0, 1.0, 1.0],
                [(1.0 - red.r - red.g) / red.g, (1.0 - green.r - green.g) / green.g, (1.0 - blue.r - blue.g) / blue.g],
            ];
            let w = XYZ {
                x: (white.x / white.y) * 100.0,
                y: 100.0,
                z: ((1.0 - white.x - white.y) / white.y) * 100.0,
            };
            // Solve P * S = W for scaling factors using Cramer's rule
            let det = p[0][0] * (p[1][1] * p[2][2] - p[1][2] * p[2][1])
                - p[0][1] * (p[1][0] * p[2][2] - p[1][2] * p[2][0])
                + p[0][2] * (p[1][0] * p[2][1] - p[1][1] * p[2][0]);
            if det.abs() < 1e-12 {
                // Fallback to sRGB if singular
                return rgb.to_xyz_srgb();
            }
            let det_sx = w.x * (p[1][1] * p[2][2] - p[1][2] * p[2][1])
                - p[0][1] * (w.y * p[2][2] - p[1][2] * w.z)
                + p[0][2] * (w.y * p[2][1] - p[1][1] * w.z);
            let det_sy = p[0][0] * (w.y * p[2][2] - p[1][2] * w.z)
                - w.x * (p[1][0] * p[2][2] - p[1][2] * p[2][0])
                + p[0][2] * (p[1][0] * w.z - w.y * p[2][0]);
            let det_sz = p[0][0] * (p[1][1] * w.z - w.y * p[2][1])
                - p[0][1] * (p[1][0] * w.z - w.y * p[2][0])
                + w.x * (p[1][0] * p[2][1] - p[1][1] * p[2][0]);
            let sx = det_sx / det;
            let sy = det_sy / det;
            let sz = det_sz / det;
            XYZ {
                x: p[0][0] * sx * rgb.r + p[0][1] * sy * rgb.g + p[0][2] * sz * rgb.b,
                y: p[1][0] * sx * rgb.r + p[1][1] * sy * rgb.g + p[1][2] * sz * rgb.b,
                z: p[2][0] * sx * rgb.r + p[2][1] * sy * rgb.g + p[2][2] * sz * rgb.b,
            }
        }
    }
}

/// Compute DeltaE 2000 between a measured XYZ and a target RGB for a given white point.
fn compute_delta_e_for_patch(measured: &XYZ, target_rgb: &RGB, target_space: &TargetSpace, white_point: &WhitePoint) -> f64 {
    let target_xyz = rgb_to_target_xyz(target_rgb, target_space);
    let wp = match white_point {
        WhitePoint::D65 => color_science::types::WhitePoint::D65,
        WhitePoint::D50 => color_science::types::WhitePoint::D50,
        WhitePoint::Dci => color_science::types::WhitePoint::Custom { x: 0.314, y: 0.351 },
        WhitePoint::Custom(xyz) => color_science::types::WhitePoint::Custom { x: xyz.x, y: xyz.y },
    };
    let lab1 = measured.to_lab(wp);
    let lab2 = target_xyz.to_lab(wp);
    delta_e_2000(&lab1, &lab2)
}

/// Generate a patch set from a preset name and optional custom colors.
pub fn generate_manual_patches(preset: &str, custom: Option<Vec<RGB>>) -> Vec<ManualPatch> {
    let mut patches = Vec::new();
    match preset {
        "grayscale" | "full" => {
            for i in 0..=10 {
                let level = i as f64 / 10.0;
                patches.push(ManualPatch::new(RGB { r: level, g: level, b: level }, "grayscale"));
            }
        }
        _ => {}
    }
    if preset == "primaries" || preset == "full" {
        patches.push(ManualPatch::new(RGB { r: 1.0, g: 0.0, b: 0.0 }, "primary"));
        patches.push(ManualPatch::new(RGB { r: 0.0, g: 1.0, b: 0.0 }, "primary"));
        patches.push(ManualPatch::new(RGB { r: 0.0, g: 0.0, b: 1.0 }, "primary"));
    }
    if preset == "secondaries" || preset == "full" {
        patches.push(ManualPatch::new(RGB { r: 0.0, g: 1.0, b: 1.0 }, "secondary"));
        patches.push(ManualPatch::new(RGB { r: 1.0, g: 0.0, b: 1.0 }, "secondary"));
        patches.push(ManualPatch::new(RGB { r: 1.0, g: 1.0, b: 0.0 }, "secondary"));
    }
    if preset == "full" {
        patches.push(ManualPatch::new(RGB { r: 1.0, g: 1.0, b: 1.0 }, "white"));
    }
    if preset == "custom" {
        if let Some(colors) = custom {
            for rgb in colors {
                patches.push(ManualPatch::new(rgb, "custom"));
            }
        }
    }
    patches
}

/// Manual calibration flow: user-driven patch-by-patch measurement
/// with live dE feedback and optional partial LUT generation.
pub struct ManualCalibrationFlow {
    pub config: SessionConfig,
    pub state: ManualCalibrationState,
    pub patches: Vec<ManualPatch>,
    pub current_patch: usize,
    pub session_id: String,
}

impl ManualCalibrationFlow {
    pub fn new(config: SessionConfig, session_id: String) -> Self {
        Self {
            config,
            state: ManualCalibrationState::Idle,
            patches: Vec::new(),
            current_patch: 0,
            session_id,
        }
    }

    pub fn start(
        &mut self,
        meter: &mut dyn Meter,
        display: &mut dyn DisplayController,
        pattern_gen: &mut dyn PatternGenerator,
        events: &EventChannel,
    ) -> Result<(), CalibrationError> {
        self.state = ManualCalibrationState::Connecting;

        meter.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "meter".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected { device: "meter".to_string() });

        display.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "display".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected { device: "display".to_string() });

        pattern_gen.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "pattern_gen".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected { device: "pattern_gen".to_string() });

        let preset = if self.config.manual_patches.is_some() {
            "custom"
        } else {
            "full"
        };
        let custom = self.config.manual_patches.clone();
        self.patches = generate_manual_patches(preset, custom);
        self.current_patch = 0;
        self.state = ManualCalibrationState::Ready;

        events.send(CalibrationEvent::ManualStateChanged {
            state: "ready".to_string(),
            current_patch: 0,
            total_patches: self.patches.len(),
        });

        Ok(())
    }

    pub fn measure_current(
        &mut self,
        meter: &mut dyn Meter,
        pattern_gen: &mut dyn PatternGenerator,
        events: &EventChannel,
    ) -> Result<(), CalibrationError> {
        if self.patches.is_empty() {
            return Err(CalibrationError::InvalidConfig("No patches to measure".to_string()));
        }
        if self.current_patch >= self.patches.len() {
            return Err(CalibrationError::InvalidConfig("All patches already measured".to_string()));
        }

        let patch = &self.patches[self.current_patch];
        let rgb = patch.target_rgb;
        let patch_name = format!("{} {}", patch.patch_type, self.current_patch + 1);

        self.state = ManualCalibrationState::Measuring;
        events.send(CalibrationEvent::ManualStateChanged {
            state: "measuring".to_string(),
            current_patch: self.current_patch,
            total_patches: self.patches.len(),
        });

        pattern_gen.display_patch(&rgb).map_err(|e| CalibrationError::MeasurementFailed {
            patch_index: self.current_patch,
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::ManualPatchDisplayed {
            patch_index: self.current_patch,
            patch_name: patch_name.clone(),
            rgb,
        });

        if self.config.settle_time_ms > 0 {
            thread::sleep(Duration::from_millis(self.config.settle_time_ms));
        }

        let stats = MeasurementLoop::measure_sync(
            || meter.read_xyz(500).unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 }),
            self.config.reads_per_patch,
            self.config.stability_threshold,
        );

        let de = compute_delta_e_for_patch(
            &stats.mean,
            &rgb,
            &self.config.target_space,
            &self.config.white_point,
        );

        self.patches[self.current_patch].measured_xyz = Some(stats.mean);
        self.patches[self.current_patch].delta_e = Some(de);
        self.state = ManualCalibrationState::Measured;

        events.send(CalibrationEvent::ManualPatchMeasured {
            patch_index: self.current_patch,
            patch_name,
            target_rgb: rgb,
            measured_xyz: stats.mean,
            delta_e: de,
        });
        events.send(CalibrationEvent::ManualStateChanged {
            state: "measured".to_string(),
            current_patch: self.current_patch,
            total_patches: self.patches.len(),
        });

        Ok(())
    }

    pub fn next(&mut self, events: &EventChannel) -> Result<(), CalibrationError> {
        if self.current_patch + 1 >= self.patches.len() {
            return Err(CalibrationError::InvalidConfig("Already at last patch".to_string()));
        }
        self.current_patch += 1;
        self.state = ManualCalibrationState::Ready;
        events.send(CalibrationEvent::ManualStateChanged {
            state: "ready".to_string(),
            current_patch: self.current_patch,
            total_patches: self.patches.len(),
        });
        Ok(())
    }

    pub fn prev(&mut self, events: &EventChannel) -> Result<(), CalibrationError> {
        if self.current_patch == 0 {
            return Err(CalibrationError::InvalidConfig("Already at first patch".to_string()));
        }
        self.current_patch -= 1;
        self.state = ManualCalibrationState::Ready;
        events.send(CalibrationEvent::ManualStateChanged {
            state: "ready".to_string(),
            current_patch: self.current_patch,
            total_patches: self.patches.len(),
        });
        Ok(())
    }

    pub fn skip(&mut self, events: &EventChannel) -> Result<(), CalibrationError> {
        if self.patches.is_empty() || self.current_patch >= self.patches.len() {
            return Err(CalibrationError::InvalidConfig("No current patch to skip".to_string()));
        }
        let patch = &self.patches[self.current_patch];
        let patch_name = format!("{} {}", patch.patch_type, self.current_patch + 1);
        self.patches[self.current_patch].skipped = true;
        events.send(CalibrationEvent::ManualPatchSkipped {
            patch_index: self.current_patch,
            patch_name,
        });
        if self.current_patch + 1 < self.patches.len() {
            self.next(events)
        } else {
            Ok(())
        }
    }

    pub fn finish(
        &mut self,
        display: &mut dyn DisplayController,
        events: &EventChannel,
        apply_corrections: bool,
    ) -> Result<(), CalibrationError> {
        self.state = ManualCalibrationState::GeneratingLut;
        events.send(CalibrationEvent::ManualStateChanged {
            state: "generating_lut".to_string(),
            current_patch: self.current_patch,
            total_patches: self.patches.len(),
        });

        let measured: Vec<(&RGB, &XYZ)> = self.patches.iter()
            .filter(|p| p.measured_xyz.is_some() && !p.skipped)
            .map(|p| (&p.target_rgb, p.measured_xyz.as_ref().unwrap()))
            .collect();

        let lut_generated = if measured.len() >= 2 && apply_corrections {
            // Generate a minimal 1D LUT from grayscale patches if available
            let grayscale: Vec<(RGB, XYZ)> = measured.iter()
                .filter(|(rgb, _)| (rgb.r - rgb.g).abs() < 0.001 && (rgb.g - rgb.b).abs() < 0.001)
                .map(|(rgb, xyz)| (**rgb, **xyz))
                .collect();

            if grayscale.len() >= 2 {
                let analysis = calibration_autocal::greyscale::GreyscaleAnalyzer::analyze(
                    &grayscale,
                    &self.config.target_space,
                    &self.config.white_point,
                ).map_err(|e| CalibrationError::Analysis(e.to_string()))?;
                let lut = calibration_autocal::lut::Lut1DGenerator::from_corrections(
                    &analysis.per_channel_corrections,
                    256,
                );
                display.upload_1d_lut(&lut).map_err(|e| CalibrationError::DisplayUpload(e.to_string()))?;
                true
            } else {
                false
            }
        } else {
            false
        };

        let measured_count = self.patches.iter().filter(|p| p.measured_xyz.is_some() && !p.skipped).count();
        let skipped_count = self.patches.iter().filter(|p| p.skipped).count();

        self.state = ManualCalibrationState::Finished;
        events.send(CalibrationEvent::ManualCalibrationComplete {
            session_id: self.session_id.clone(),
            measured_patches: measured_count,
            skipped_patches: skipped_count,
            lut_generated,
        });
        events.send(CalibrationEvent::ManualStateChanged {
            state: "finished".to_string(),
            current_patch: self.current_patch,
            total_patches: self.patches.len(),
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calibration_core::state::ToneCurve;
    use hal::mocks::FakePatternGenerator;
    use hal::traits::Meter;
    use hal::error::MeterError;

    struct ManualTestMeter {
        connected: bool,
        call_count: usize,
    }

    impl ManualTestMeter {
        fn new() -> Self {
            Self { connected: false, call_count: 0 }
        }
    }

    impl Meter for ManualTestMeter {
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
            self.call_count += 1;
            Ok(XYZ { x: 50.0 + self.call_count as f64, y: 60.0 + self.call_count as f64, z: 40.0 + self.call_count as f64 })
        }
        fn model(&self) -> &str {
            "ManualTestMeter"
        }
    }

    #[test]
    fn test_generate_manual_patches_grayscale() {
        let patches = generate_manual_patches("grayscale", None);
        assert_eq!(patches.len(), 11);
        assert_eq!(patches[0].patch_type, "grayscale");
        assert!((patches[5].target_rgb.r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_generate_manual_patches_full() {
        let patches = generate_manual_patches("full", None);
        assert_eq!(patches.len(), 18); // 11 grayscale + 3 primaries + 3 secondaries + 1 white
        assert_eq!(patches[11].patch_type, "primary");
        assert_eq!(patches[14].patch_type, "secondary");
        assert_eq!(patches[17].patch_type, "white");
    }

    #[test]
    fn test_generate_manual_patches_custom() {
        let custom = vec![RGB { r: 0.5, g: 0.2, b: 0.8 }];
        let patches = generate_manual_patches("custom", Some(custom));
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].patch_type, "custom");
    }

    #[test]
    fn test_manual_flow_start_and_measure() {
        let config = SessionConfig {
            name: "Manual Test".to_string(),
            target_space: TargetSpace::Bt709,
            tone_curve: ToneCurve::Gamma(2.2),
            white_point: WhitePoint::D65,
            patch_count: 0,
            reads_per_patch: 1,
            settle_time_ms: 0,
            stability_threshold: None,
            tier: calibration_core::state::CalibrationTier::GrayscaleOnly,
            manual_patches: Some(vec![
                RGB { r: 1.0, g: 0.0, b: 0.0 },
                RGB { r: 0.0, g: 1.0, b: 0.0 },
            ]),
        };
        let mut flow = ManualCalibrationFlow::new(config, "test-manual".to_string());
        let mut meter = ManualTestMeter::new();
        let mut display = hal::mocks::FakeDisplayController::default();
        let mut pattern_gen = FakePatternGenerator::default();
        let events = EventChannel::new(16);

        flow.start(&mut meter, &mut display, &mut pattern_gen, &events).unwrap();
        assert_eq!(flow.patches.len(), 2);
        assert_eq!(flow.state, ManualCalibrationState::Ready);

        flow.measure_current(&mut meter, &mut pattern_gen, &events).unwrap();
        assert_eq!(flow.state, ManualCalibrationState::Measured);
        assert!(flow.patches[0].measured_xyz.is_some());
        assert!(flow.patches[0].delta_e.is_some());

        flow.next(&events).unwrap();
        assert_eq!(flow.current_patch, 1);

        flow.measure_current(&mut meter, &mut pattern_gen, &events).unwrap();
        assert!(flow.patches[1].measured_xyz.is_some());

        flow.finish(&mut display, &events, false).unwrap();
        assert_eq!(flow.state, ManualCalibrationState::Finished);
    }

    #[test]
    fn test_manual_flow_skip() {
        let config = SessionConfig {
            name: "Manual Test".to_string(),
            target_space: TargetSpace::Bt709,
            tone_curve: ToneCurve::Gamma(2.2),
            white_point: WhitePoint::D65,
            patch_count: 0,
            reads_per_patch: 1,
            settle_time_ms: 0,
            stability_threshold: None,
            tier: calibration_core::state::CalibrationTier::GrayscaleOnly,
            manual_patches: Some(vec![RGB { r: 1.0, g: 0.0, b: 0.0 }]),
        };
        let mut flow = ManualCalibrationFlow::new(config, "test-skip".to_string());
        let mut meter = ManualTestMeter::new();
        let mut display = hal::mocks::FakeDisplayController::default();
        let mut pattern_gen = FakePatternGenerator::default();
        let events = EventChannel::new(16);

        flow.start(&mut meter, &mut display, &mut pattern_gen, &events).unwrap();
        flow.skip(&events).unwrap();
        assert!(flow.patches[0].skipped);
        assert_eq!(flow.state, ManualCalibrationState::Ready);
    }

    #[test]
    fn test_rgb_to_target_xyz_bt709() {
        let rgb = RGB { r: 1.0, g: 0.0, b: 0.0 };
        let xyz = rgb_to_target_xyz(&rgb, &TargetSpace::Bt709);
        // sRGB red should be around X=41.2, Y=21.3, Z=1.9
        assert!((xyz.x - 41.2).abs() < 1.0);
        assert!((xyz.y - 21.3).abs() < 1.0);
        assert!((xyz.z - 1.9).abs() < 1.0);
    }
}
