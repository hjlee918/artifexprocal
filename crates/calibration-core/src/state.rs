use color_science::types::{RGB, XYZ};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetSpace {
    Bt709,
    Bt2020,
    DciP3,
    Custom { red: RGB, green: RGB, blue: RGB, white: XYZ },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToneCurve {
    Gamma(f64),
    Bt1886,
    Pq,
    Hlg,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WhitePoint {
    D65,
    D50,
    Dci,
    Custom(XYZ),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CalibrationTier {
    GrayscaleOnly,
    GrayscalePlus3D,
    Full3D,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionConfig {
    pub name: String,
    pub target_space: TargetSpace,
    pub tone_curve: ToneCurve,
    pub white_point: WhitePoint,
    pub patch_count: usize,
    pub reads_per_patch: usize,
    pub settle_time_ms: u64,
    pub stability_threshold: Option<f64>,
    pub tier: CalibrationTier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationState {
    Idle,
    Connecting,
    Connected,
    Measuring { current_patch: usize, total_patches: usize },
    Paused { at_patch: usize },
    Analyzing,
    ComputingLut,
    Uploading,
    Finished,
    Error(CalibrationError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationEvent {
    DeviceConnected { device: String },
    PatchDisplayed { patch_index: usize, rgb: RGB },
    ReadingsComplete { patch_index: usize, xyz: XYZ, std_dev: XYZ },
    ProgressUpdated { current: usize, total: usize },
    AnalysisComplete { gamma: f64, max_de: f64, white_balance_errors: Vec<f64> },
    LutGenerated { size: usize },
    CorrectionsUploaded,
    SessionComplete { session_id: String },
    Error(CalibrationError),
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum CalibrationError {
    #[error("Device connection failed: {device} - {reason}")]
    ConnectionFailed { device: String, reason: String },

    #[error("Measurement failed at patch {patch_index}: {reason}")]
    MeasurementFailed { patch_index: usize, reason: String },

    #[error("Meter read failed: {0}")]
    MeterRead(String),

    #[error("Display upload failed: {0}")]
    DisplayUpload(String),

    #[error("Analysis failed: {0}")]
    Analysis(String),

    #[error("Session paused by user")]
    Paused,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
