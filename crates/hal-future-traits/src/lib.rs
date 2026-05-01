//! hal-future-traits — Design sketches for Phase 3+ hardware abstractions.
//!
//! WARNING: Everything in this crate is speculative. These traits and
//! types were extracted from premature design work in the `hal` crate.
//! They will change significantly when real display and pattern generator
//! drivers are implemented. Do not depend on them outside of design docs.

pub mod display;
pub mod pattern_gen;
pub mod types;

pub use display::{DisplayController, DisplayError};
pub use pattern_gen::{PatternGenError, PatternGenerator};
pub use types::{
    CalibrationData, Lut1D, Lut3D, Patch, PictureMode, WhiteBalance,
};
