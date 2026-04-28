use crate::ipc::models::{
    CalibrationProgress, CalibrationStateEvent, DeviceStatusEvent, ErrorEvent,
    ProfilingProgress,
};
use calibration_core::state::CalibrationEvent;
use tauri::{AppHandle, Emitter};

pub fn emit_device_status_changed(
    app: &AppHandle,
    device_id: String,
    device_type: String,
    connected: bool,
    info: String,
) {
    if let Err(e) = app.emit(
        "device-status-changed",
        DeviceStatusEvent {
            device_id,
            device_type,
            connected,
            info,
        },
    ) {
        eprintln!("Failed to emit device-status-changed: {}", e);
    }
}

pub fn emit_calibration_state_changed(
    app: &AppHandle,
    old_state: crate::ipc::models::CalibrationState,
    new_state: crate::ipc::models::CalibrationState,
    message: String,
) {
    if let Err(e) = app.emit(
        "calibration-state-changed",
        CalibrationStateEvent {
            old_state,
            new_state,
            message,
        },
    ) {
        eprintln!("Failed to emit calibration-state-changed: {}", e);
    }
}

pub fn emit_error_occurred(
    app: &AppHandle,
    severity: String,
    message: String,
    source: String,
) {
    if let Err(e) = app.emit(
        "error-occurred",
        ErrorEvent {
            severity,
            message,
            source,
        },
    ) {
        eprintln!("Failed to emit error-occurred: {}", e);
    }
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
    if let Err(e) = app.emit(
        "calibration-progress",
        CalibrationProgress {
            session_id,
            current_patch,
            total_patches,
            patch_name,
            yxy,
            stable,
        },
    ) {
        eprintln!("Failed to emit calibration-progress: {}", e);
    }
}

pub fn emit_analysis_complete(
    app: &AppHandle,
    session_id: String,
    gamma: f64,
    max_de: f64,
    avg_de: f64,
    white_balance_errors: Vec<f64>,
) {
    if let Err(e) = app.emit(
        "analysis-complete",
        serde_json::json!({
            "session_id": session_id,
            "gamma": gamma,
            "max_de": max_de,
            "avg_de": avg_de,
            "white_balance_errors": white_balance_errors,
        }),
    ) {
        eprintln!("Failed to emit analysis-complete: {}", e);
    }
}

pub fn emit_lut_uploaded(app: &AppHandle, session_id: String) {
    if let Err(e) = app.emit("lut-uploaded", serde_json::json!({ "session_id": session_id })) {
        eprintln!("Failed to emit lut-uploaded: {}", e);
    }
}

pub fn emit_lut3d_generated(
    app: &AppHandle,
    session_id: String,
    size: usize,
    format: String,
) {
    if let Err(e) = app.emit(
        "lut3d-generated",
        serde_json::json!({
            "session_id": session_id,
            "size": size,
            "format": format,
        }),
    ) {
        eprintln!("Failed to emit lut3d-generated: {}", e);
    }
}

pub fn emit_verification_complete(
    app: &AppHandle,
    session_id: String,
    pre_de: Vec<f64>,
    post_de: Vec<f64>,
) {
    if let Err(e) = app.emit(
        "verification-complete",
        serde_json::json!({
            "session_id": session_id,
            "pre_de": pre_de,
            "post_de": post_de,
        }),
    ) {
        eprintln!("Failed to emit verification-complete: {}", e);
    }
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
    if let Err(e) = app.emit(
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
    ) {
        eprintln!("Failed to emit profiling-progress: {}", e);
    }
}

pub fn emit_profiling_complete(
    app: &AppHandle,
    session_id: String,
    correction_matrix: [[f64; 3]; 3],
    accuracy_estimate: f64,
) {
    if let Err(e) = app.emit(
        "profiling-complete",
        serde_json::json!({
            "session_id": session_id,
            "correction_matrix": correction_matrix,
            "accuracy_estimate": accuracy_estimate,
        }),
    ) {
        eprintln!("Failed to emit profiling-complete: {}", e);
    }
}

pub fn emit_engine_event(
    app: &AppHandle,
    session_id: &str,
    event: CalibrationEvent,
) {
    match event {
        CalibrationEvent::DeviceConnected { device } => {
            emit_device_status_changed(app, device.clone(), "device".into(), true, format!("{} connected", device));
        }
        CalibrationEvent::PatchDisplayed { patch_index, .. } => {
            emit_calibration_progress(
                app,
                session_id.to_string(),
                patch_index,
                0,
                format!("Patch {}", patch_index),
                None,
                false,
            );
        }
        CalibrationEvent::ReadingsComplete { patch_index, xyz, .. } => {
            let yxy = color_science::types::XYZ { x: xyz.x, y: xyz.y, z: xyz.z }.to_xyy();
            emit_calibration_progress(
                app,
                session_id.to_string(),
                patch_index,
                0,
                format!("Patch {}", patch_index),
                Some((yxy.Y, yxy.x, yxy.y)),
                true,
            );
        }
        CalibrationEvent::ProgressUpdated { current, total } => {
            emit_calibration_progress(
                app,
                session_id.to_string(),
                current,
                total,
                format!("Patch {}", current),
                None,
                false,
            );
        }
        CalibrationEvent::AnalysisComplete { gamma, max_de, white_balance_errors } => {
            let avg_de = white_balance_errors.iter().sum::<f64>() / white_balance_errors.len().max(1) as f64;
            emit_analysis_complete(
                app,
                session_id.to_string(),
                gamma,
                max_de,
                avg_de,
                white_balance_errors,
            );
        }
        CalibrationEvent::LutGenerated { size } => {
            emit_lut_uploaded(app, session_id.to_string());
        }
        CalibrationEvent::CorrectionsUploaded => {
            emit_lut_uploaded(app, session_id.to_string());
        }
        CalibrationEvent::SessionComplete { .. } => {
            emit_verification_complete(app, session_id.to_string(), vec![], vec![]);
        }
        CalibrationEvent::Error(e) => {
            emit_error_occurred(app, "error".into(), e.to_string(), "engine".into());
        }
    }
}
