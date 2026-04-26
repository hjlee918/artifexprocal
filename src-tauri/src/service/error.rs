use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalibrationError {
    #[error("Meter {0} not found. Is it plugged in?")]
    MeterNotFound(String),
    #[error("Display {0} not found. Check network connection.")]
    DisplayNotFound(String),
    #[error("Meter {0} is already in use by another process.")]
    MeterInUse(String),
    #[error("Operation failed: {0}")]
    Internal(String),
}
