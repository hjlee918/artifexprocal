#[cfg(test)]
mod tests {
    const EXTRA_EXPORTS: &str = r#"

// ─── Named command exports (backwards compatibility) ───────────────────────

export const {
	computeDeltaE,
	computeXyy,
	getAppState,
	connectMeter,
	disconnectMeter,
	connectDisplay,
	disconnectDisplay,
	getDeviceInventory,
	startCalibration,
	abortCalibration,
	startProfiling,
	abortProfiling,
	getSpectralLocus,
	getTargetGamut,
	generate3dLut,
	exportLut,
	listSessions,
	getSessionDetail,
	exportSessionData,
	generateReport,
} = commands;

// ─── Event constants (manually maintained) ─────────────────────────────────

export const EVENT_DEVICE_STATUS_CHANGED = "device-status-changed" as const;
export const EVENT_CALIBRATION_STATE_CHANGED = "calibration-state-changed" as const;
export const EVENT_ERROR_OCCURRED = "error-occurred" as const;
export const EVENT_CALIBRATION_PROGRESS = "calibration-progress" as const;
export const EVENT_ANALYSIS_COMPLETE = "analysis-complete" as const;
export const EVENT_LUT_UPLOADED = "lut-uploaded" as const;
export const EVENT_VERIFICATION_COMPLETE = "verification-complete" as const;
export const EVENT_PROFILING_PROGRESS = "profiling-progress" as const;
export const EVENT_LUT3D_GENERATED = "lut3d-generated" as const;
export const EVENT_LUT3D_DATA = "lut3d-data" as const;
export const EVENT_PROFILING_COMPLETE = "profiling-complete" as const;

export type EventName =
	| typeof EVENT_DEVICE_STATUS_CHANGED
	| typeof EVENT_CALIBRATION_STATE_CHANGED
	| typeof EVENT_ERROR_OCCURRED
	| typeof EVENT_CALIBRATION_PROGRESS
	| typeof EVENT_ANALYSIS_COMPLETE
	| typeof EVENT_LUT_UPLOADED
	| typeof EVENT_VERIFICATION_COMPLETE
	| typeof EVENT_PROFILING_PROGRESS
	| typeof EVENT_PROFILING_COMPLETE
	| typeof EVENT_LUT3D_GENERATED
	| typeof EVENT_LUT3D_DATA;
"#;

    #[test]
    fn export_typescript_bindings() {
        let builder = tauri_specta::Builder::<tauri::Wry>::new()
            .error_handling(tauri_specta::ErrorHandlingMode::Throw)
            .commands(tauri_specta::collect_commands![
                crate::compute_delta_e,
                crate::compute_xyy,
                crate::ipc::commands::get_app_state,
                crate::ipc::commands::connect_meter,
                crate::ipc::commands::disconnect_meter,
                crate::ipc::commands::connect_display,
                crate::ipc::commands::disconnect_display,
                crate::ipc::commands::get_device_inventory,
                crate::ipc::commands::start_calibration,
                crate::ipc::commands::abort_calibration,
                crate::ipc::commands::start_profiling,
                crate::ipc::commands::abort_profiling,
                crate::ipc::commands::get_spectral_locus,
                crate::ipc::commands::get_target_gamut,
                crate::ipc::commands::generate_3d_lut,
                crate::ipc::commands::export_lut,
                crate::ipc::commands::list_sessions,
                crate::ipc::commands::get_session_detail,
                crate::ipc::commands::export_session_data,
                crate::ipc::commands::generate_report,
            ]);

        let path = "../src/bindings.ts";
        builder
            .export(
                specta_typescript::Typescript::default(),
                path,
            )
            .expect("Failed to export typescript bindings");

        std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, EXTRA_EXPORTS.as_bytes()))
            .expect("Failed to append extra exports");
    }
}
