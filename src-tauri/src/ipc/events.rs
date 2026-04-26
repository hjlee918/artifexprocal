use crate::ipc::models::{
    CalibrationProgress, CalibrationStateEvent, DeviceStatusEvent, ErrorEvent,
    ProfilingProgress,
};
use tauri::{AppHandle, Emitter};

pub fn emit_device_status_changed(
    app: &AppHandle,
    device_id: String,
    device_type: String,
    connected: bool,
    info: String,
) {
    let _ = app.emit(
        "device-status-changed",
        DeviceStatusEvent {
            device_id,
            device_type,
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

pub fn emit_calibration_progress(
    app: &AppHandle,
    session_id: String,
    current_patch: usize,
    total_patches: usize,
    patch_name: String,
    yxy: Option<(f64, f64, f64)>,
    stable: bool,
) {
    let _ = app.emit(
        "calibration-progress",
        CalibrationProgress {
            session_id,
            current_patch,
            total_patches,
            patch_name,
            yxy,
            stable,
        },
    );
}

pub fn emit_analysis_complete(
    app: &AppHandle,
    session_id: String,
    gamma: f64,
    max_de: f64,
    avg_de: f64,
    white_balance_errors: Vec<f64>,
) {
    let _ = app.emit(
        "analysis-complete",
        serde_json::json!({
            "session_id": session_id,
            "gamma": gamma,
            "max_de": max_de,
            "avg_de": avg_de,
            "white_balance_errors": white_balance_errors,
        }),
    );
}

pub fn emit_lut_uploaded(app: &AppHandle, session_id: String) {
    let _ = app.emit("lut-uploaded", serde_json::json!({ "session_id": session_id }));
}

pub fn emit_verification_complete(
    app: &AppHandle,
    session_id: String,
    pre_de: Vec<f64>,
    post_de: Vec<f64>,
) {
    let _ = app.emit(
        "verification-complete",
        serde_json::json!({
            "session_id": session_id,
            "pre_de": pre_de,
            "post_de": post_de,
        }),
    );
}

pub fn emit_profiling_progress(
    app: &AppHandle,
    session_id: String,
    current_patch: usize,
    total_patches: usize,
    patch_name: String,
    reference_xyz: (f64, f64, f64),
    meter_xyz: (f64, f64, f64),
    delta_e: f64,
) {
    let _ = app.emit(
        "profiling-progress",
        ProfilingProgress {
            session_id,
            current_patch,
            total_patches,
            patch_name,
            reference_xyz,
            meter_xyz,
            delta_e,
        },
    );
}

pub fn emit_profiling_complete(
    app: &AppHandle,
    session_id: String,
    correction_matrix: [[f64; 3]; 3],
    accuracy_estimate: f64,
) {
    let _ = app.emit(
        "profiling-complete",
        serde_json::json!({
            "session_id": session_id,
            "correction_matrix": correction_matrix,
            "accuracy_estimate": accuracy_estimate,
        }),
    );
}
