//! Display controller trait — for LG OLED, Sony projector, etc.

use crate::types::PictureMode;
use std::fmt;

/// Error type for display operations.
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayError {
    ConnectionFailed,
    NotInCalibrationMode,
    UploadFailed,
    InvalidPictureMode,
    Timeout,
    Other(String),
}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayError::ConnectionFailed => write!(f, "Failed to connect to display"),
            DisplayError::NotInCalibrationMode => {
                write!(f, "Display not in calibration mode")
            }
            DisplayError::UploadFailed => write!(f, "LUT upload failed"),
            DisplayError::InvalidPictureMode => write!(f, "Invalid picture mode"),
            DisplayError::Timeout => write!(f, "Display operation timed out"),
            DisplayError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DisplayError {}

/// Abstract interface for display controllers.
pub trait DisplayController: Send {
    /// Self-test: verify connectivity.
    fn probe(&mut self) -> Result<bool, DisplayError>;

    /// Enter calibration mode for a specific picture mode.
    fn start_calibration(
        &mut self,
        picture_mode: PictureMode,
    ) -> Result<(), DisplayError>;

    /// Exit calibration mode.
    fn end_calibration(&mut self) -> Result<(), DisplayError>;

    /// Upload a 1D LUT.
    fn upload_1d_lut(
        &mut self,
        picture_mode: PictureMode,
        lut: &crate::types::Lut1D,
    ) -> Result<(), DisplayError>;

    /// Upload a 3D LUT for a specific color space.
    fn upload_3d_lut(
        &mut self,
        picture_mode: PictureMode,
        color_space: &str,
        lut: &crate::types::Lut3D,
    ) -> Result<(), DisplayError>;

    /// Disconnect from the display.
    fn disconnect(&mut self) -> Result<(), DisplayError> {
        Ok(())
    }
}
