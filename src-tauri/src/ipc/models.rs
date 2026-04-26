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

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct DeviceStatusEvent {
    pub device_id: String,
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
