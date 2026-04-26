use crate::ipc::models::{
    AppState, CalibrationState, DeviceInfo, DisplayInfo, MeterInfo,
};
use crate::service::CalibrationService;
use tauri::State;

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
    service: State<'_, CalibrationService>,
    meter_id: String,
) -> Result<MeterInfo, String> {
    let info = service
        .connect_meter(&meter_id)
        .map_err(|e| e.to_string())?;
    Ok(info)
}

#[tauri::command]
#[specta::specta]
pub fn disconnect_meter(
    service: State<'_, CalibrationService>,
    meter_id: String,
) -> Result<(), String> {
    service
        .disconnect_meter(&meter_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn connect_display(
    service: State<'_, CalibrationService>,
    display_id: String,
) -> Result<DisplayInfo, String> {
    let info = service
        .connect_display(&display_id)
        .map_err(|e| e.to_string())?;
    Ok(info)
}

#[tauri::command]
#[specta::specta]
pub fn disconnect_display(
    service: State<'_, CalibrationService>,
    display_id: String,
) -> Result<(), String> {
    service
        .disconnect_display(&display_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_device_inventory(
    service: State<'_, CalibrationService>,
) -> Result<Vec<DeviceInfo>, String> {
    Ok(service.get_device_inventory())
}
