use crate::ipc::events;
use crate::ipc::models::{
    AppState, CalibrationState, DeviceInfo, DisplayInfo, MeterInfo,
};
use crate::service::CalibrationService;
use std::time::Duration;
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub fn get_app_state(service: State<'_, CalibrationService>) -> Result<AppState, String> {
    Ok(AppState {
        meters: service.get_meter_info(),
        displays: service.get_display_info(),
        calibration_state: service.get_state(),
        last_error: None,
    })
}

#[tauri::command]
#[specta::specta]
pub fn connect_meter(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    meter_id: String,
) -> Result<MeterInfo, String> {
    let info = service
        .connect_meter(&meter_id)
        .map_err(|e| e.to_string())?;
    events::emit_device_status_changed(
        &app,
        info.id.clone(),
        "meter".to_string(),
        true,
        info.name.clone(),
    );
    Ok(info)
}

#[tauri::command]
#[specta::specta]
pub fn disconnect_meter(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    meter_id: String,
) -> Result<(), String> {
    service
        .disconnect_meter(&meter_id)
        .map_err(|e| e.to_string())?;
    events::emit_device_status_changed(
        &app,
        meter_id,
        "meter".to_string(),
        false,
        "Meter disconnected".to_string(),
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn connect_display(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    display_id: String,
) -> Result<DisplayInfo, String> {
    let info = service
        .connect_display(&display_id)
        .map_err(|e| e.to_string())?;
    events::emit_device_status_changed(
        &app,
        info.id.clone(),
        "display".to_string(),
        true,
        info.name.clone(),
    );
    Ok(info)
}

#[tauri::command]
#[specta::specta]
pub fn disconnect_display(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    display_id: String,
) -> Result<(), String> {
    service
        .disconnect_display(&display_id)
        .map_err(|e| e.to_string())?;
    events::emit_device_status_changed(
        &app,
        display_id,
        "display".to_string(),
        false,
        "Display disconnected".to_string(),
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_device_inventory(
    service: State<'_, CalibrationService>,
) -> Result<Vec<DeviceInfo>, String> {
    Ok(service.get_device_inventory())
}

#[tauri::command]
#[specta::specta]
pub fn start_calibration(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    config: crate::ipc::models::SessionConfigDto,
) -> Result<String, String> {
    let session_config = calibration_core::state::SessionConfig {
        name: config.name,
        target_space: match config.target_space.as_str() {
            "Rec.2020" => calibration_core::state::TargetSpace::Bt2020,
            "DCI-P3" => calibration_core::state::TargetSpace::DciP3,
            "Rec.709" => calibration_core::state::TargetSpace::Bt709,
            _ => return Err(format!("Invalid target_space: {}", config.target_space)),
        },
        tone_curve: match config.tone_curve.as_str() {
            "Gamma 2.2" => calibration_core::state::ToneCurve::Gamma(2.2),
            "Gamma 2.4" => calibration_core::state::ToneCurve::Gamma(2.4),
            "BT.1886" => calibration_core::state::ToneCurve::Bt1886,
            "PQ" => calibration_core::state::ToneCurve::Pq,
            "HLG" => calibration_core::state::ToneCurve::Hlg,
            _ => return Err(format!("Invalid tone_curve: {}", config.tone_curve)),
        },
        white_point: match config.white_point.as_str() {
            "D50" => calibration_core::state::WhitePoint::D50,
            "DCI" => calibration_core::state::WhitePoint::Dci,
            "D65" => calibration_core::state::WhitePoint::D65,
            _ => return Err(format!("Invalid white_point: {}", config.white_point)),
        },
        patch_count: config.patch_count,
        reads_per_patch: config.reads_per_patch,
        settle_time_ms: config.settle_time_ms,
        stability_threshold: config.stability_threshold,
    };

    let session_id = service
        .start_calibration_session(session_config)
        .map_err(|e| e.to_string())?;

    // Spawn calibration in blocking thread (placeholder — full integration in Task 5)
    let app_clone = app.clone();
    let patch_count = config.patch_count;
    let session_id_clone = session_id.clone();
    std::thread::spawn(move || {
        // Emit a dummy progress event after 1s for testing
        std::thread::sleep(Duration::from_secs(1));
        crate::ipc::events::emit_calibration_progress(
            &app_clone,
            session_id_clone,
            0,
            patch_count,
            "0% Black".to_string(),
            Some((0.02, 0.3125, 0.3290)),
            true,
        );
    });

    Ok(session_id)
}

#[tauri::command]
#[specta::specta]
pub fn abort_calibration(
    service: State<'_, CalibrationService>,
    session_id: String,
) -> Result<(), String> {
    if service.get_active_session_id() != Some(session_id.clone()) {
        return Err(crate::service::error::CalibrationError::SessionNotFound(session_id).to_string());
    }
    service.end_session();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn start_profiling(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    meter_id: String,
    _reference_meter_id: String,
    _display_id: String,
    _config: crate::ipc::models::ProfilingConfig,
) -> Result<String, String> {
    let session_id = format!("prof-{}", uuid::Uuid::new_v4());
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        crate::ipc::events::emit_profiling_progress(
            &app_clone,
            session_id_clone,
            0,
            20,
            "Primary Red".to_string(),
            (45.2, 25.1, 12.3),
            (44.8, 24.9, 12.1),
            0.35,
        );
    });
    Ok(session_id)
}

#[tauri::command]
#[specta::specta]
pub fn abort_profiling(
    _service: State<'_, CalibrationService>,
    _session_id: String,
) -> Result<(), String> {
    Ok(())
}
