//! # color-science
//!
//! Core colorimetric math for ArtifexProCal: colorspace conversions,
//! DeltaE calculations, CCT/Duv, blackbody synthesis, and CIE 1931
//! 2° standard observer data.
//!
//! ## Phase 1 Features
//! - XYZ ↔ xyY ↔ Lab ↔ LCh ↔ u′v′ conversions
//! - CIE DE2000 (validated against Sharma et al. reference dataset)
//! - CIE DE76
//! - CCT and Duv (Ohno 2013, Robertson 1968 cross-check in tests)
//! - Blackbody SPD synthesis + tristimulus integration
//! - CIE 1931 2° observer tabulation (360–830 nm, 5 nm)
//! - `MeasurementResult` — universal colorimetric data contract

pub mod blackbody;
pub mod cct;
pub mod cie1931;
pub mod conversion;
pub mod delta_e;
pub mod measurement;
pub mod types;

// Re-export commonly used types at crate root.
pub use types::{ICtCp, Lab, LCh, Rgb, RgbSpace, UvPrime, WhitePoint, XyY, Xyz};
pub use measurement::MeasurementResult;
