//! hal-meters — Meter driver implementations.

use color_science::types::Xyz;
use hal::meter::{MeasurementMode, Meter, MeterError};

/// A fake meter for integration testing that returns deterministic values.
pub struct FakeMeter {
    mode: MeasurementMode,
}

impl FakeMeter {
    pub fn new() -> Self {
        Self {
            mode: MeasurementMode::Emissive,
        }
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
        match self.mode {
            MeasurementMode::Emissive => Ok(Xyz {
                x: 95.047,
                y: 100.0,
                z: 108.883,
            }),
            _ => Ok(Xyz {
                x: 95.047,
                y: 100.0,
                z: 108.883,
            }),
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
