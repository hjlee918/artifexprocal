use calibration_core::state::{CalibrationState, CalibrationEvent, SessionConfig, CalibrationError, CalibrationTier};
use calibration_core::patch::{PatchSet, PatchStrategy};
use calibration_core::measure::MeasurementLoop;
use calibration_storage::schema::Storage;
use calibration_storage::session_store::SessionStore;
use calibration_storage::reading_store::ReadingStore;
use calibration_autocal::greyscale::GreyscaleAnalyzer;
use calibration_autocal::lut::Lut1DGenerator;
use calibration_autocal::lut3d::Lut3DEngine;
use calibration_autocal::patch3d::OptimizedPatchSetGenerator;
use hal::traits::{Meter, DisplayController, PatternGenerator};
use hal::types::{Lut3D, RGBGain};
use color_science::types::{RGB, XYZ};
use crate::events::EventChannel;
use std::time::Duration;
use std::thread;

pub struct Lut3DAutoCalFlow {
    pub config: SessionConfig,
    pub state: CalibrationState,
    pub patches: Option<PatchSet>,
    pub current_patch: usize,
    pub lut_1d: Option<hal::types::Lut1D>,
    pub lut_3d: Option<Lut3D>,
}

impl Lut3DAutoCalFlow {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            state: CalibrationState::Idle,
            patches: None,
            current_patch: 0,
            lut_1d: None,
            lut_3d: None,
        }
    }

    pub fn start(&mut self) -> Result<(), CalibrationError> {
        self.state = CalibrationState::Connecting;
        Ok(())
    }

    pub fn generate_patches(&mut self) {
        let strategy = match self.config.tier {
            CalibrationTier::GrayscaleOnly => PatchStrategy::Grayscale(self.config.patch_count),
            CalibrationTier::GrayscalePlus3D => PatchStrategy::OptimizedSubset {
                grayscale_count: self.config.patch_count,
                color_count: 180,
            },
            CalibrationTier::Full3D => PatchStrategy::OptimizedSubset {
                grayscale_count: 33,
                color_count: 600,
            },
        };
        let patches = OptimizedPatchSetGenerator::generate(strategy);
        self.patches = Some(patches);
        self.current_patch = 0;
    }

    pub fn run_sync(
        &mut self,
        meter: &mut dyn Meter,
        display: &mut dyn DisplayController,
        pattern_gen: &mut dyn PatternGenerator,
        storage: &Storage,
        events: &EventChannel,
    ) -> Result<(), CalibrationError> {
        let session_store = SessionStore::new(&storage.conn);
        let reading_store = ReadingStore::new(&storage.conn);

        // Connect devices (same as GreyscaleAutoCalFlow)
        self.state = CalibrationState::Connecting;
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

        self.state = CalibrationState::Connected;

        // Create session
        let session_id = session_store.create(&self.config)
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;
        session_store.update_state(&session_id, "measuring")
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;

        // Generate patches
        self.generate_patches();
        let total = self.patches.as_ref().unwrap().len();
        events.send(CalibrationEvent::ProgressUpdated { current: 0, total });

        // Measurement loop
        let mut readings: Vec<(RGB, XYZ)> = Vec::with_capacity(total);

        for i in 0..total {
            if let CalibrationState::Paused { at_patch } = self.state {
                if at_patch == i {
                    return Err(CalibrationError::Paused);
                }
            }

            let patch = self.patches.as_ref().unwrap().get(i);
            let rgb = patch.target_rgb;

            pattern_gen.display_patch(&rgb).map_err(|e| CalibrationError::MeasurementFailed {
                patch_index: i,
                reason: e.to_string(),
            })?;
            events.send(CalibrationEvent::PatchDisplayed { patch_index: i, rgb });

            if self.config.settle_time_ms > 0 {
                thread::sleep(Duration::from_millis(self.config.settle_time_ms));
            }

            self.state = CalibrationState::Measuring { current_patch: i, total_patches: total };

            let stats = MeasurementLoop::measure_sync(
                || meter.read_xyz(500).unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 }),
                self.config.reads_per_patch,
                self.config.stability_threshold,
            );

            for ri in 0..self.config.reads_per_patch {
                reading_store.save(&session_id, i, ri, &stats.mean, "cal")
                    .map_err(|e| CalibrationError::MeasurementFailed {
                        patch_index: i,
                        reason: e.to_string(),
                    })?;
            }

            events.send(CalibrationEvent::ReadingsComplete {
                patch_index: i,
                xyz: stats.mean,
                std_dev: stats.std_dev,
            });

            readings.push((rgb, stats.mean));
            events.send(CalibrationEvent::ProgressUpdated { current: i + 1, total });
        }

        // Analysis (grayscale portion only)
        self.state = CalibrationState::Analyzing;
        let grayscale_readings: Vec<(RGB, XYZ)> = readings.iter()
            .filter(|(rgb, _)| (rgb.r - rgb.g).abs() < 0.001 && (rgb.g - rgb.b).abs() < 0.001)
            .cloned()
            .collect();

        let analysis = GreyscaleAnalyzer::analyze(
            &grayscale_readings,
            &self.config.target_space,
            &self.config.white_point,
        ).map_err(CalibrationError::Analysis)?;

        events.send(CalibrationEvent::AnalysisComplete {
            gamma: analysis.gamma,
            max_de: analysis.max_de,
            white_balance_errors: analysis.white_balance_errors.clone(),
        });

        // 1D LUT generation
        self.state = CalibrationState::ComputingLut;
        let lut_1d = Lut1DGenerator::from_corrections(&analysis.per_channel_corrections, 256);
        events.send(CalibrationEvent::LutGenerated { size: lut_1d.size });
        self.lut_1d = Some(lut_1d);

        // 3D LUT generation (if tier is not GrayscaleOnly)
        if self.config.tier != CalibrationTier::GrayscaleOnly {
            let lut_3d_33 = Lut3DEngine::compute(&readings, 33, &self.config.target_space)
                .map_err(CalibrationError::Analysis)?;

            // Downsample to 17³ if needed
            let lut_3d = if display.model().contains("Alpha 7") {
                Lut3DEngine::downsample_33_to_17(&lut_3d_33)
                    .map_err(CalibrationError::Analysis)?
            } else {
                lut_3d_33
            };

            events.send(CalibrationEvent::LutGenerated { size: lut_3d.size });
            self.lut_3d = Some(lut_3d);
        }

        // Upload
        self.state = CalibrationState::Uploading;
        if let Some(ref lut_3d) = self.lut_3d {
            display.upload_3d_lut(lut_3d).map_err(|e| CalibrationError::DisplayUpload(e.to_string()))?;
        }
        if let Some(ref lut_1d) = self.lut_1d {
            display.upload_1d_lut(lut_1d).map_err(|e| CalibrationError::DisplayUpload(e.to_string()))?;
        }

        let wb_gains = RGBGain { r: 1.0, g: 1.0, b: 1.0 };
        display.set_white_balance(wb_gains).map_err(|e| CalibrationError::DisplayUpload(e.to_string()))?;

        events.send(CalibrationEvent::CorrectionsUploaded);

        // Complete
        session_store.update_state(&session_id, "finished")
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;
        self.state = CalibrationState::Finished;
        events.send(CalibrationEvent::SessionComplete { session_id });

        Ok(())
    }
}
