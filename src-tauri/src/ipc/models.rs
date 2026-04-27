use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct MeterInfo {
    pub id: String,
    pub name: String,
    pub serial: Option<String>,
    pub connected: bool,
    pub capabilities: Vec<String>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub model: String,
    pub connected: bool,
    pub picture_mode: Option<String>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug, Default)]
pub enum CalibrationState {
    #[default]
    Idle,
    Connecting,
    Measuring,
    GeneratingLut,
    Uploading,
    Verifying,
    Error,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct AppState {
    pub meters: Vec<MeterInfo>,
    pub displays: Vec<DisplayInfo>,
    pub calibration_state: CalibrationState,
    pub last_error: Option<String>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub available: bool,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug, Copy)]
pub struct Chromaticity {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct GamutDto {
    pub red: Chromaticity,
    pub green: Chromaticity,
    pub blue: Chromaticity,
    pub white: Chromaticity,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct DeviceStatusEvent {
    pub device_id: String,
    pub device_type: String,
    pub connected: bool,
    pub info: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct CalibrationStateEvent {
    pub old_state: CalibrationState,
    pub new_state: CalibrationState,
    pub message: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct ErrorEvent {
    pub severity: String,
    pub message: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct SessionConfigDto {
    pub name: String,
    pub target_space: String,
    pub tone_curve: String,
    pub white_point: String,
    pub patch_count: usize,
    pub reads_per_patch: usize,
    pub settle_time_ms: u64,
    pub stability_threshold: Option<f64>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct CalibrationProgress {
    pub session_id: String,
    pub current_patch: usize,
    pub total_patches: usize,
    pub patch_name: String,
    pub yxy: Option<(f64, f64, f64)>,
    pub stable: bool,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct ProfilingConfig {
    pub patch_set: String,
    pub patch_scale: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct ProfilingProgress {
    pub session_id: String,
    pub current_patch: usize,
    pub total_patches: usize,
    pub patch_name: String,
    pub reference_xyz: (f64, f64, f64),
    pub meter_xyz: (f64, f64, f64),
    pub delta_e: f64,
}
