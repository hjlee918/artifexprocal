pub mod bindings_export;
pub mod ipc;
pub mod service;

use color_science::types::{Lab, XYZ};
use color_science::delta_e::delta_e_2000;

#[tauri::command]
fn compute_delta_e(l1: f64, a1: f64, b1: f64, l2: f64, a2: f64, b2: f64) -> f64 {
    let lab1 = Lab { L: l1, a: a1, b: b1 };
    let lab2 = Lab { L: l2, a: a2, b: b2 };
    delta_e_2000(&lab1, &lab2)
}

#[tauri::command]
fn compute_xyy(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let xyz = XYZ { x, y, z };
    let xyy = xyz.to_xyy();
    (xyy.x, xyy.y, xyy.Y)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(service::state::CalibrationService::new())
        .invoke_handler(tauri::generate_handler![
            compute_delta_e,
            compute_xyy,
            ipc::commands::get_app_state,
            ipc::commands::connect_meter,
            ipc::commands::disconnect_meter,
            ipc::commands::connect_display,
            ipc::commands::disconnect_display,
            ipc::commands::get_device_inventory,
            ipc::commands::start_calibration,
            ipc::commands::abort_calibration,
            ipc::commands::start_profiling,
            ipc::commands::abort_profiling,
            ipc::commands::get_spectral_locus,
            ipc::commands::get_target_gamut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
