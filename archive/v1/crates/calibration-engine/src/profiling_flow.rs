use calibration_core::state::{CalibrationEvent, CalibrationError};
use calibration_core::measure::MeasurementLoop;
use calibration_core::patch::{Patch, PatchSet};
use calibration_storage::profiling_session_store::ProfilingSessionStore;
use calibration_storage::schema::Storage;
use color_science::delta_e::delta_e_2000;
use color_science::types::{RGB, XYZ, WhitePoint};
use hal::traits::{Meter, PatternGenerator};
use hal_meters::profiling::generate_correction_matrix;
use crate::events::EventChannel;
use std::time::Duration;
use std::thread;

/// A profiling session configuration.
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    pub patch_count: usize,
    pub reads_per_patch: usize,
    pub settle_time_ms: u64,
    pub stability_threshold: Option<f64>,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            patch_count: 20,
            reads_per_patch: 3,
            settle_time_ms: 5000,
            stability_threshold: Some(0.5),
        }
    }
}

/// Generate a standard 20-patch profiling set:
/// - 10 grayscale steps
/// - 3 primaries (R, G, B at 100%)
/// - 3 secondaries (C, M, Y at 100%)
/// - 4 skin-tone / near-neutral colors
fn generate_profiling_patches() -> PatchSet {
    let mut patches = Vec::with_capacity(20);

    // 10-step grayscale
    for i in 0..10 {
        let level = i as f64 / 9.0;
        patches.push(Patch::new(RGB { r: level, g: level, b: level }));
    }

    // Primaries
    patches.push(Patch::new(RGB { r: 1.0, g: 0.0, b: 0.0 }));
    patches.push(Patch::new(RGB { r: 0.0, g: 1.0, b: 0.0 }));
    patches.push(Patch::new(RGB { r: 0.0, g: 0.0, b: 1.0 }));

    // Secondaries
    patches.push(Patch::new(RGB { r: 0.0, g: 1.0, b: 1.0 }));
    patches.push(Patch::new(RGB { r: 1.0, g: 0.0, b: 1.0 }));
    patches.push(Patch::new(RGB { r: 1.0, g: 1.0, b: 0.0 }));

    // Skin tones / near-neutrals
    patches.push(Patch::new(RGB { r: 0.72, g: 0.52, b: 0.40 }));
    patches.push(Patch::new(RGB { r: 0.55, g: 0.35, b: 0.25 }));
    patches.push(Patch::new(RGB { r: 0.85, g: 0.65, b: 0.55 }));
    patches.push(Patch::new(RGB { r: 0.45, g: 0.40, b: 0.38 }));

    PatchSet { patches }
}

fn patch_name(index: usize, rgb: &RGB) -> String {
    if (rgb.r - rgb.g).abs() < 0.001 && (rgb.g - rgb.b).abs() < 0.001 {
        format!("Gray {:.0}%", rgb.r * 100.0)
    } else if rgb.r > 0.9 && rgb.g < 0.1 && rgb.b < 0.1 {
        "Primary Red".to_string()
    } else if rgb.r < 0.1 && rgb.g > 0.9 && rgb.b < 0.1 {
        "Primary Green".to_string()
    } else if rgb.r < 0.1 && rgb.g < 0.1 && rgb.b > 0.9 {
        "Primary Blue".to_string()
    } else if rgb.r < 0.1 && rgb.g > 0.9 && rgb.b > 0.9 {
        "Secondary Cyan".to_string()
    } else if rgb.r > 0.9 && rgb.g < 0.1 && rgb.b > 0.9 {
        "Secondary Magenta".to_string()
    } else if rgb.r > 0.9 && rgb.g > 0.9 && rgb.b < 0.1 {
        "Secondary Yellow".to_string()
    } else {
        format!("Patch {}", index + 1)
    }
}

/// Profiling flow that measures a display with both a reference spectrophotometer
/// and a field colorimeter, then computes a correction matrix.
pub struct ProfilingFlow {
    pub config: ProfilingConfig,
    pub patches: Option<PatchSet>,
    pub current_patch: usize,
    pub reference_readings: Vec<XYZ>,
    pub meter_readings: Vec<XYZ>,
    pub correction_matrix: Option<[[f64; 3]; 3]>,
    pub accuracy: Option<f64>,
}

impl ProfilingFlow {
    pub fn new(config: ProfilingConfig) -> Self {
        Self {
            config,
            patches: None,
            current_patch: 0,
            reference_readings: Vec::new(),
            meter_readings: Vec::new(),
            correction_matrix: None,
            accuracy: None,
        }
    }

    pub fn generate_patches(&mut self) {
        self.patches = Some(generate_profiling_patches());
        self.current_patch = 0;
        self.reference_readings.clear();
        self.meter_readings.clear();
        self.correction_matrix = None;
        self.accuracy = None;
    }

    pub fn run_sync(
        &mut self,
        session_id: &str,
        field_meter_id: &str,
        reference_meter_id: &str,
        display_id: Option<&str>,
        reference_meter: &mut dyn Meter,
        field_meter: &mut dyn Meter,
        pattern_gen: &mut dyn PatternGenerator,
        storage: &Storage,
        events: &EventChannel,
    ) -> Result<(), CalibrationError> {
        let store = ProfilingSessionStore::new(&storage.conn);
        store.create(session_id, session_id, field_meter_id, reference_meter_id, display_id)
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;

        // Connect devices
        reference_meter.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "reference_meter".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected {
            device: "reference_meter".to_string(),
        });

        field_meter.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "field_meter".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected {
            device: "field_meter".to_string(),
        });

        pattern_gen.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "pattern_gen".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected {
            device: "pattern_gen".to_string(),
        });

        // Generate patches
        self.generate_patches();
        let total = self.patches.as_ref().unwrap().len();
        events.send(CalibrationEvent::ProgressUpdated { current: 0, total });

        let wp = WhitePoint::D65;

        for i in 0..total {
            let patch = self.patches.as_ref().unwrap().get(i);
            let rgb = patch.target_rgb;
            let name = patch_name(i, &rgb);

            pattern_gen.display_patch(&rgb).map_err(|e| CalibrationError::MeasurementFailed {
                patch_index: i,
                reason: e.to_string(),
            })?;
            events.send(CalibrationEvent::PatchDisplayed { patch_index: i, rgb });

            if self.config.settle_time_ms > 0 {
                thread::sleep(Duration::from_millis(self.config.settle_time_ms));
            }

            // Read reference meter
            let ref_stats = MeasurementLoop::measure_sync(
                || reference_meter.read_xyz(500).unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 }),
                self.config.reads_per_patch,
                self.config.stability_threshold,
            );

            // Read field meter
            let field_stats = MeasurementLoop::measure_sync(
                || field_meter.read_xyz(500).unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 }),
                self.config.reads_per_patch,
                self.config.stability_threshold,
            );

            let ref_xyz = ref_stats.mean;
            let field_xyz = field_stats.mean;

            // Compute uncorrected dE2000
            let ref_lab = ref_xyz.to_lab(wp);
            let field_lab = field_xyz.to_lab(wp);
            let de = delta_e_2000(&ref_lab, &field_lab);

            store.save_reading(session_id, i, &rgb, &ref_xyz, &field_xyz, de)
                .map_err(|e| CalibrationError::MeasurementFailed {
                    patch_index: i,
                    reason: e.to_string(),
                })?;

            events.send(CalibrationEvent::ProfilingProgress {
                patch_index: i,
                total_patches: total,
                patch_name: name,
                reference_xyz: ref_xyz,
                meter_xyz: field_xyz,
                delta_e: de,
            });

            self.reference_readings.push(ref_xyz);
            self.meter_readings.push(field_xyz);
            self.current_patch = i + 1;

            events.send(CalibrationEvent::ProgressUpdated { current: i + 1, total });
        }

        // Compute correction matrix
        let matrix = generate_correction_matrix(&self.meter_readings, &self.reference_readings)
            .map_err(|e| CalibrationError::Analysis(e.to_string()))?;

        let accuracy = matrix.accuracy(&self.meter_readings, &self.reference_readings);
        let matrix_array = matrix.m;

        store.save_result(session_id, &matrix_array, accuracy, total)
            .map_err(|e| CalibrationError::Analysis(e.to_string()))?;
        store.update_state(session_id, "finished")
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;

        self.correction_matrix = Some(matrix_array);
        self.accuracy = Some(accuracy);

        events.send(CalibrationEvent::ProfilingComplete {
            correction_matrix: matrix_array,
            accuracy_estimate: accuracy,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hal::mocks::FakePatternGenerator;
    use hal::traits::Meter;
    use hal::error::MeterError;

    /// Mock meter that returns varied XYZ based on an internal counter,
    /// simulating different patches producing different readings.
    struct ProfilingMeter {
        connected: bool,
        call_count: usize,
        scale: f64,
    }

    impl ProfilingMeter {
        fn new(scale: f64) -> Self {
            Self { connected: false, call_count: 0, scale }
        }
    }

    impl Meter for ProfilingMeter {
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
            let patch_index = self.call_count;
            // Produce varied readings: each patch gets a distinct XYZ
            let r = ((patch_index * 7 + 3) % 101) as f64 / 100.0 * 80.0 + 10.0;
            let g = ((patch_index * 11 + 5) % 101) as f64 / 100.0 * 90.0 + 5.0;
            let b = ((patch_index * 13 + 7) % 101) as f64 / 100.0 * 70.0 + 15.0;
            self.call_count += 1;
            Ok(XYZ { x: r * self.scale, y: g * self.scale, z: b * self.scale })
        }
        fn model(&self) -> &str {
            "ProfilingMeter"
        }
    }

    #[test]
    fn test_profiling_flow_generates_20_patches() {
        let config = ProfilingConfig::default();
        let mut flow = ProfilingFlow::new(config);
        flow.generate_patches();
        assert_eq!(flow.patches.as_ref().unwrap().len(), 20);
    }

    #[test]
    fn test_profiling_flow_perfect_meters() {
        let config = ProfilingConfig {
            patch_count: 20,
            reads_per_patch: 1,
            settle_time_ms: 0,
            stability_threshold: None,
        };
        let mut flow = ProfilingFlow::new(config);

        let mut reference_meter = ProfilingMeter::new(1.0);
        let mut field_meter = ProfilingMeter::new(1.0);
        let mut pattern_gen = FakePatternGenerator::default();

        let storage = Storage::new_in_memory().unwrap();
        let events = EventChannel::new(16);

        flow.run_sync(
            "test-session", "meter1", "meter2", None,
            &mut reference_meter, &mut field_meter, &mut pattern_gen, &storage, &events,
        )
        .unwrap();

        assert_eq!(flow.reference_readings.len(), 20);
        assert_eq!(flow.meter_readings.len(), 20);
        assert!(flow.correction_matrix.is_some());
        let matrix = flow.correction_matrix.unwrap();
        // With identical readings, matrix should be approximately identity
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (matrix[i][j] - expected).abs() < 1e-3,
                    "m[{i}][{j}] = {} != {expected}",
                    matrix[i][j]
                );
            }
        }
        assert!(flow.accuracy.unwrap() < 1e-3);
    }

    #[test]
    fn test_profiling_flow_scaled_meter() {
        let config = ProfilingConfig {
            patch_count: 20,
            reads_per_patch: 1,
            settle_time_ms: 0,
            stability_threshold: None,
        };
        let mut flow = ProfilingFlow::new(config);

        // Field meter reads 2x the reference
        let mut reference_meter = ProfilingMeter::new(1.0);
        let mut field_meter = ProfilingMeter::new(2.0);
        let mut pattern_gen = FakePatternGenerator::default();

        let storage = Storage::new_in_memory().unwrap();
        let events = EventChannel::new(16);

        flow.run_sync(
            "test-session", "meter1", "meter2", None,
            &mut reference_meter, &mut field_meter, &mut pattern_gen, &storage, &events,
        )
        .unwrap();

        let matrix = flow.correction_matrix.unwrap();
        // With 2x readings, matrix should be approximately 0.5 * I
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 0.5 } else { 0.0 };
                assert!(
                    (matrix[i][j] - expected).abs() < 1e-3,
                    "m[{i}][{j}] = {} != {expected}",
                    matrix[i][j]
                );
            }
        }
    }
}
