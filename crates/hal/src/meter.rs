//! Meter trait — colorimeter and spectrophotometer abstraction.

use color_science::types::Xyz;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error type for meter operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    SequenceExhausted,
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
            MeterError::SequenceExhausted => write!(f, "Measurement sequence exhausted"),
            MeterError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl MeterError {
    /// Returns true if this error is transient and may resolve on retry.
    ///
    /// Note: `SequenceExhausted` returns `false` because it is a clean
    /// terminal condition, not an error. Call sites that need to distinguish
    /// terminal exhaustion from fatal errors should match on the variant
    /// directly after checking `!is_transient()`.
    pub fn is_transient(&self) -> bool {
        match self {
            MeterError::Disconnected => true,
            MeterError::Timeout => true,
            MeterError::Saturated => true,
            MeterError::NoOutput => true,
            MeterError::JoinError(_) => true,
            MeterError::Other(_) => true,
            MeterError::SequenceExhausted => false,
            MeterError::UnlockFailed => false,
            MeterError::NotInstalled => false,
            MeterError::CalibrationRequired => false,
            MeterError::InvalidMode => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_errors_classified() {
        assert!(MeterError::Disconnected.is_transient());
        assert!(MeterError::Timeout.is_transient());
        assert!(MeterError::Saturated.is_transient());
        assert!(MeterError::NoOutput.is_transient());
        assert!(MeterError::JoinError("x".into()).is_transient());
        assert!(MeterError::Other("x".into()).is_transient());
    }

    #[test]
    fn fatal_errors_not_transient() {
        assert!(!MeterError::UnlockFailed.is_transient());
        assert!(!MeterError::NotInstalled.is_transient());
        assert!(!MeterError::CalibrationRequired.is_transient());
        assert!(!MeterError::InvalidMode.is_transient());
    }

    #[test]
    fn sequence_exhausted_not_transient() {
        assert!(!MeterError::SequenceExhausted.is_transient());
    }
}

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
