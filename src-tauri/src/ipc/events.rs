use crate::ipc::models::{
    CalibrationStateEvent, DeviceStatusEvent, ErrorEvent,
};
use tauri::{AppHandle, Emitter};

pub fn emit_device_status_changed(
    app: &AppHandle,
    device_id: String,
    connected: bool,
    info: String,
) {
    let _ = app.emit(
        "device-status-changed",
        DeviceStatusEvent {
            device_id,
            connected,
            info,
        },
    );
}

pub fn emit_calibration_state_changed(
    app: &AppHandle,
    old_state: crate::ipc::models::CalibrationState,
    new_state: crate::ipc::models::CalibrationState,
    message: String,
) {
    let _ = app.emit(
        "calibration-state-changed",
        CalibrationStateEvent {
            old_state,
            new_state,
            message,
        },
    );
}

pub fn emit_error_occurred(
    app: &AppHandle,
    severity: String,
    message: String,
    source: String,
) {
    let _ = app.emit(
        "error-occurred",
        ErrorEvent {
            severity,
            message,
            source,
        },
    );
}
