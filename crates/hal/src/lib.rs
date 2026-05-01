//! hal — Hardware abstraction layer traits.
//!
//! Phase 1 scope: Meter trait only. DisplayController and PatternGenerator
//! are intentionally excluded — they belong in `hal-future-traits` as
//! design sketches until Phase 3+.

pub mod meter;

pub use meter::{MeasurementMode, Meter, MeterError};
