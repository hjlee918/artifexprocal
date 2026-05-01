//! hal — Hardware abstraction layer traits.

pub mod display;
pub mod meter;
pub mod pattern_gen;
pub mod types;

pub use display::{DisplayController, DisplayError};
pub use meter::{MeasurementMode, Meter, MeterError};
pub use pattern_gen::{PatternGenError, PatternGenerator};
pub use types::{
    CalibrationData, Lut1D, Lut3D, Patch, PictureMode, WhiteBalance,
};
