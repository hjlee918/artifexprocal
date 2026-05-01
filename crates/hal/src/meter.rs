//! Meter trait — colorimeter and spectrophotometer abstraction.

use color_science::types::Xyz;
use std::fmt;

/// Error type for meter operations.
#[derive(Debug, Clone, PartialEq)]
pub enum MeterError {
    Disconnected,
    Timeout,
    Saturated,
    UnlockFailed,
    NotInstalled,
    NoOutput,
    JoinError(String),
    CalibrationRequired,
    InvalidMode,
    Other(String),
}

impl fmt::Display for MeterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeterError::Disconnected => write!(f, "Meter disconnected"),
            MeterError::Timeout => write!(f, "Measurement timed out"),
            MeterError::Saturated => write!(f, "Signal saturated"),
            MeterError::UnlockFailed => write!(f, "Failed to unlock meter"),
            MeterError::NotInstalled => write!(f, "Driver not installed"),
            MeterError::NoOutput => write!(f, "No output from meter"),
            MeterError::JoinError(msg) => write!(f, "Task join error: {}", msg),
            MeterError::CalibrationRequired => write!(f, "Meter requires calibration"),
            MeterError::InvalidMode => write!(f, "Invalid measurement mode"),
            MeterError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MeterError {}

/// Measurement mode for meters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasurementMode {
    Emissive,
    Ambient,
    Flash,
    Telephoto,
    Reflective,
    Transmissive,
}

/// A colorimeter or spectrophotometer.
pub trait Meter: Send {
    /// Self-test: verify connectivity without taking a measurement.
    fn probe(&mut self) -> Result<bool, MeterError>;

    /// Take a single XYZ reading.
    fn read_xyz(&mut self) -> Result<Xyz, MeterError>;

    /// Set the measurement mode.
    fn set_mode(&mut self, _mode: MeasurementMode) -> Result<(), MeterError> {
        Ok(())
    }

    /// Disconnect from the instrument.
    fn disconnect(&mut self) -> Result<(), MeterError> {
        Ok(())
    }
}
