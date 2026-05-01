//! hal-meters — Meter driver implementations.

use color_science::types::{Xyz, D65};
use hal::meter::{MeasurementMode, Meter, MeterError};
use serde::{Deserialize, Serialize};

/// Configuration for FakeMeter's output behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FakeMeterConfig {
    /// Return a fixed XYZ on every read.
    Hardcoded(Xyz),
    /// Replay a sequence of XYZ values.
    Sequence {
        values: Vec<Xyz>,
        /// If true, wrap to index 0 after exhausting `values`.
        /// If false, return `MeterError::SequenceExhausted`.
        loop_at_end: bool,
    },
    /// Synthesize blackbody XYZ along the Planckian locus.
    PlanckianSweep {
        start_cct: f64,
        end_cct: f64,
        steps: usize,
        /// Target luminance Y in cd/m².
        target_luminance: f64,
        /// If true, wrap to step 0 after exhausting the sweep.
        /// If false, return `MeterError::SequenceExhausted`.
        loop_at_end: bool,
    },
}

/// A fake meter for integration testing that returns deterministic values.
pub struct FakeMeter {
    config: FakeMeterConfig,
    mode: MeasurementMode,
    sequence_index: usize,
    planckian_step_index: usize,
}

impl FakeMeter {
    pub fn new() -> Self {
        Self::with_config(FakeMeterConfig::Hardcoded(D65))
            .expect("Hardcoded(D65) is always valid")
    }

    pub fn with_config(config: FakeMeterConfig) -> Result<Self, MeterError> {
        if let FakeMeterConfig::PlanckianSweep { steps, .. } = &config {
            if *steps == 0 {
                return Err(MeterError::Other(
                    "PlanckianSweep steps must be > 0".to_string(),
                ));
            }
        }
        Ok(Self {
            config,
            mode: MeasurementMode::Emissive,
            sequence_index: 0,
            planckian_step_index: 0,
        })
    }
}

impl Default for FakeMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl Meter for FakeMeter {
    fn probe(&mut self) -> Result<bool, MeterError> {
        Ok(true)
    }

    fn read_xyz(&mut self) -> Result<Xyz, MeterError> {
        match &self.config {
            FakeMeterConfig::Hardcoded(xyz) => Ok(*xyz),
            FakeMeterConfig::Sequence { values, loop_at_end } => {
                if self.sequence_index >= values.len() {
                    if *loop_at_end {
                        self.sequence_index = 0;
                    } else {
                        return Err(MeterError::SequenceExhausted);
                    }
                }
                let xyz = values[self.sequence_index];
                self.sequence_index += 1;
                Ok(xyz)
            }
            FakeMeterConfig::PlanckianSweep {
                start_cct,
                end_cct,
                steps,
                target_luminance,
                loop_at_end,
            } => {
                if self.planckian_step_index >= *steps {
                    if *loop_at_end {
                        self.planckian_step_index = 0;
                    } else {
                        return Err(MeterError::SequenceExhausted);
                    }
                }
                let cct = if *steps == 1 {
                    *start_cct
                } else {
                    start_cct
                        + self.planckian_step_index as f64 * (end_cct - start_cct)
                            / (steps - 1) as f64
                };
                // Use the SPD integration path (not the locus-table path) so that
                // the self-consistency test is a true cross-check against the Ohno
                // 2013 tabulated locus rather than a round-trip on the same data.
                let xyz = color_science::blackbody::blackbody_xyz(cct, *target_luminance);
                self.planckian_step_index += 1;
                Ok(xyz)
            }
        }
    }

    fn set_mode(&mut self, mode: MeasurementMode) -> Result<(), MeterError> {
        self.mode = mode;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), MeterError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardcoded_returns_fixed_xyz() {
        let mut meter = FakeMeter::new();
        let a = meter.read_xyz().unwrap();
        let b = meter.read_xyz().unwrap();
        assert_eq!(a, b);
        assert_eq!(a, D65);
    }

    #[test]
    fn sequence_non_looping_exhaustion() {
        let xyz1 = Xyz { x: 10.0, y: 20.0, z: 30.0 };
        let xyz2 = Xyz { x: 11.0, y: 21.0, z: 31.0 };
        let config = FakeMeterConfig::Sequence {
            values: vec![xyz1, xyz2],
            loop_at_end: false,
        };
        let mut meter = FakeMeter::with_config(config).unwrap();
        assert_eq!(meter.read_xyz().unwrap(), xyz1);
        assert_eq!(meter.read_xyz().unwrap(), xyz2);
        assert_eq!(meter.read_xyz(), Err(MeterError::SequenceExhausted));
    }

    #[test]
    fn sequence_looping_wraparound() {
        let xyz1 = Xyz { x: 10.0, y: 20.0, z: 30.0 };
        let xyz2 = Xyz { x: 11.0, y: 21.0, z: 31.0 };
        let config = FakeMeterConfig::Sequence {
            values: vec![xyz1, xyz2],
            loop_at_end: true,
        };
        let mut meter = FakeMeter::with_config(config).unwrap();
        assert_eq!(meter.read_xyz().unwrap(), xyz1);
        assert_eq!(meter.read_xyz().unwrap(), xyz2);
        assert_eq!(meter.read_xyz().unwrap(), xyz1); // wrap
        assert_eq!(meter.read_xyz().unwrap(), xyz2);
    }

    #[test]
    fn planckian_sweep_at_d65_lands_on_locus() {
        let config = FakeMeterConfig::PlanckianSweep {
            start_cct: 6504.0,
            end_cct: 6504.0,
            steps: 1,
            target_luminance: 100.0,
            loop_at_end: false,
        };
        let mut meter = FakeMeter::with_config(config).unwrap();
        let xyz = meter.read_xyz().unwrap();
        let (_, duv) = color_science::cct::xyz_to_cct_duv(xyz);
        assert!(
            duv.abs() <= 0.0005,
            "Duv expected <= 0.0005, got {}",
            duv
        );
    }

    #[test]
    fn planckian_sweep_non_looping_exhaustion() {
        let config = FakeMeterConfig::PlanckianSweep {
            start_cct: 3000.0,
            end_cct: 4000.0,
            steps: 2,
            target_luminance: 100.0,
            loop_at_end: false,
        };
        let mut meter = FakeMeter::with_config(config).unwrap();
        let _ = meter.read_xyz().unwrap();
        let _ = meter.read_xyz().unwrap();
        assert_eq!(meter.read_xyz(), Err(MeterError::SequenceExhausted));
    }

    #[test]
    fn planckian_sweep_steps_are_distinct() {
        let config = FakeMeterConfig::PlanckianSweep {
            start_cct: 3000.0,
            end_cct: 10_000.0,
            steps: 8,
            target_luminance: 100.0,
            loop_at_end: false,
        };
        let mut meter = FakeMeter::with_config(config).unwrap();
        let mut values = Vec::new();
        for _ in 0..8 {
            values.push(meter.read_xyz().unwrap());
        }
        // All 8 values must be pairwise distinct.
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert_ne!(
                    values[i], values[j],
                    "values at index {} and {} should differ",
                    i, j
                );
            }
        }
    }

    #[test]
    fn planckian_sweep_steps_zero_rejected() {
        let config = FakeMeterConfig::PlanckianSweep {
            start_cct: 3000.0,
            end_cct: 4000.0,
            steps: 0,
            target_luminance: 100.0,
            loop_at_end: false,
        };
        let result = FakeMeter::with_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn planckian_sweep_steps_one_returns_start_cct() {
        let config = FakeMeterConfig::PlanckianSweep {
            start_cct: 5000.0,
            end_cct: 6000.0,
            steps: 1,
            target_luminance: 100.0,
            loop_at_end: false,
        };
        let mut meter = FakeMeter::with_config(config).unwrap();
        let xyz = meter.read_xyz().unwrap();
        let (cct, _) = color_science::cct::xyz_to_cct_duv(xyz);
        assert!(
            (cct - 5000.0).abs() < 50.0,
            "CCT expected ~5000K, got {}",
            cct
        );
        assert_eq!(meter.read_xyz(), Err(MeterError::SequenceExhausted));
    }
}
