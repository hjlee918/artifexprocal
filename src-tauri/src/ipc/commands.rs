use crate::ipc::events;
use crate::ipc::models::{
    AppState, DeviceInfo, DisplayInfo, MeterInfo, Lut3DInfoDto,
    SessionConfigDto, SessionDetailDto, SessionFilterDto, SessionListResponse,
    SessionSummaryDto, ComputedResultsDto, PatchReadingDto,
};
use crate::service::CalibrationService;
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
        tier: match config.tier.as_str() {
            "GrayscalePlus3D" => calibration_core::state::CalibrationTier::GrayscalePlus3D,
            "Full3D" => calibration_core::state::CalibrationTier::Full3D,
            _ => calibration_core::state::CalibrationTier::GrayscaleOnly,
        },
    };

    let session_id = service
        .start_calibration_session(session_config)
        .map_err(|e| e.to_string())?;

    service
        .run_calibration(app, session_id.clone())
        .map_err(|e| e.to_string())?;

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
    service.request_abort();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn start_profiling(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    meter_id: String,
    reference_meter_id: String,
    _display_id: String,
    _config: crate::ipc::models::ProfilingConfig,
) -> Result<String, String> {
    let session_id = format!("prof-{}", uuid::Uuid::new_v4());

    // Ensure primary (field) meter is connected
    if !service.is_meter_connected(&meter_id) {
        service.connect_meter(&meter_id).map_err(|e| e.to_string())?;
    }

    // Ensure reference meter is connected
    if !service.is_reference_meter_connected(&reference_meter_id) {
        service.connect_reference_meter(&reference_meter_id).map_err(|e| e.to_string())?;
    }

    service
        .run_profiling(app, session_id.clone())
        .map_err(|e| e.to_string())?;

    Ok(session_id)
}

#[tauri::command]
#[specta::specta]
pub fn abort_profiling(
    service: State<'_, CalibrationService>,
    _session_id: String,
) -> Result<(), String> {
    service.request_abort();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_spectral_locus(diagram: String) -> Result<Vec<(f64, f64)>, String> {
    match diagram.as_str() {
        "1931" => Ok(color_science::diagrams::SPECTRAL_LOCUS_1931.to_vec()),
        "1976" => Ok(color_science::diagrams::spectral_locus_1976().to_vec()),
        _ => Err(format!("Invalid diagram: {}", diagram)),
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_target_gamut(target_space: String) -> Result<crate::ipc::models::GamutDto, String> {
    use crate::ipc::models::Chromaticity;

    let (r, g, b, w) = match target_space.as_str() {
        "Rec.709" | "sRGB" => (
            Chromaticity { x: 0.64, y: 0.33 },
            Chromaticity { x: 0.30, y: 0.60 },
            Chromaticity { x: 0.15, y: 0.06 },
            Chromaticity { x: 0.3127, y: 0.3290 },
        ),
        "Rec.2020" => (
            Chromaticity { x: 0.708, y: 0.292 },
            Chromaticity { x: 0.170, y: 0.797 },
            Chromaticity { x: 0.131, y: 0.046 },
            Chromaticity { x: 0.3127, y: 0.3290 },
        ),
        // Note: DCI-P3 uses the DCI theater white point (0.314, 0.351), not D65.
        // Display P3 (Apple / consumer) uses D65; request "Display P3" for that.
        "DCI-P3" => (
            Chromaticity { x: 0.680, y: 0.320 },
            Chromaticity { x: 0.265, y: 0.690 },
            Chromaticity { x: 0.150, y: 0.060 },
            Chromaticity { x: 0.314, y: 0.351 },
        ),
        "Adobe RGB" => (
            Chromaticity { x: 0.640, y: 0.330 },
            Chromaticity { x: 0.210, y: 0.710 },
            Chromaticity { x: 0.150, y: 0.060 },
            Chromaticity { x: 0.3127, y: 0.3290 },
        ),
        _ => return Err(format!("Invalid target_space: {}", target_space)),
    };
    Ok(crate::ipc::models::GamutDto {
        red: r,
        green: g,
        blue: b,
        white: w,
    })
}

#[tauri::command]
#[specta::specta]
pub fn generate_3d_lut(
    service: State<'_, CalibrationService>,
    session_id: String,
) -> Result<Lut3DInfoDto, String> {
    let detail = service
        .get_session_detail(&session_id)?
        .ok_or_else(|| "Session not found".to_string())?;

    let size = detail.results.as_ref().and_then(|r| r.lut_3d_size).unwrap_or(33);

    Ok(Lut3DInfoDto {
        size,
        format: "cube".to_string(),
        file_path: None,
    })
}

#[tauri::command]
#[specta::specta]
pub fn export_lut(
    service: State<'_, CalibrationService>,
    session_id: String,
    format: String,
    path: String,
) -> Result<(), String> {
    let detail = service
        .get_session_detail(&session_id)?
        .ok_or_else(|| "Session not found".to_string())?;

    // Reconstruct readings as (RGB, XYZ) pairs for LUT computation
    let patches: Vec<(color_science::types::RGB, color_science::types::XYZ)> = detail
        .readings
        .iter()
        .map(|r| {
            let rgb = color_science::types::RGB {
                r: r.target_rgb.0,
                g: r.target_rgb.1,
                b: r.target_rgb.2,
            };
            let xyz = color_science::types::XYZ {
                x: r.measured_xyz.x,
                y: r.measured_xyz.y,
                z: r.measured_xyz.z,
            };
            (rgb, xyz)
        })
        .collect();

    if patches.is_empty() {
        return Err("No readings available for LUT export".to_string());
    }

    let lut = calibration_autocal::lut3d::Lut3DEngine::compute(
        &patches,
        33,
        &detail.config.target_space,
    )
    .map_err(|e| format!("LUT computation failed: {e}"))?;

    let mut file = std::fs::File::create(&path).map_err(|e| format!("Failed to create file: {e}"))?;

    match format.to_lowercase().as_str() {
        "cube" => calibration_autocal::export::Lut3DExporter::export_cube(&lut, &mut file,
        )
        .map_err(|e| format!("Export failed: {e}"))?,
        "3dl" => calibration_autocal::export::Lut3DExporter::export_3dl(&lut, &mut file,
        )
        .map_err(|e| format!("Export failed: {e}"))?,
        _ => return Err(format!("Unsupported LUT format: {format}")),
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_sessions(
    service: State<'_, CalibrationService>,
    filter: SessionFilterDto,
    page: usize,
    per_page: usize,
) -> Result<SessionListResponse, String> {
    let storage_filter = calibration_storage::query::SessionFilter {
        target_space: filter.target_space,
        state: filter.state,
        date_from: filter.date_from,
        date_to: filter.date_to,
        search: filter.search,
    };

    let (items, total) = service.list_sessions(storage_filter, page, per_page)?;

    let dtos: Vec<SessionSummaryDto> = items.into_iter().map(|s| SessionSummaryDto {
        id: s.id,
        name: s.name,
        created_at: s.created_at,
        ended_at: s.ended_at,
        state: s.state,
        target_space: s.target_space,
        tier: s.tier.unwrap_or_default(),
        patch_count: s.patch_count,
        gamma: s.gamma,
        max_de: s.max_de,
        avg_de: s.avg_de,
    }).collect();

    Ok(SessionListResponse { items: dtos, total })
}

#[tauri::command]
#[specta::specta]
pub fn get_session_detail(
    service: State<'_, CalibrationService>,
    session_id: String,
) -> Result<SessionDetailDto, String> {
    let detail = service.get_session_detail(&session_id)?;

    let detail = detail.ok_or_else(|| "Session not found".to_string())?;

    let results_dto = detail.results.map(|r| ComputedResultsDto {
        gamma: r.gamma,
        max_de: r.max_de,
        avg_de: r.avg_de,
        white_balance: r.white_balance,
        lut_1d_size: r.lut_1d_size,
        lut_3d_size: r.lut_3d_size,
    });

    let readings_dto: Vec<PatchReadingDto> = detail.readings.into_iter().map(|r| PatchReadingDto {
        patch_index: r.patch_index,
        target_rgb: r.target_rgb,
        measured_xyz: (r.measured_xyz.x, r.measured_xyz.y, r.measured_xyz.z),
        reading_index: r.reading_index,
        measurement_type: r.measurement_type,
    }).collect();

    Ok(SessionDetailDto {
        summary: SessionSummaryDto {
            id: detail.summary.id,
            name: detail.summary.name,
            created_at: detail.summary.created_at,
            ended_at: detail.summary.ended_at,
            state: detail.summary.state,
            target_space: detail.summary.target_space,
            tier: detail.summary.tier.unwrap_or_default(),
            patch_count: detail.summary.patch_count,
            gamma: detail.summary.gamma,
            max_de: detail.summary.max_de,
            avg_de: detail.summary.avg_de,
        },
        config: SessionConfigDto {
            name: detail.config.name,
            target_space: format!("{:?}", detail.config.target_space),
            tone_curve: format!("{:?}", detail.config.tone_curve),
            white_point: format!("{:?}", detail.config.white_point),
            patch_count: detail.config.patch_count,
            reads_per_patch: detail.config.reads_per_patch,
            settle_time_ms: detail.config.settle_time_ms,
            stability_threshold: detail.config.stability_threshold,
            tier: format!("{:?}", detail.config.tier),
        },
        readings: readings_dto,
        results: results_dto,
    })
}

#[tauri::command]
#[specta::specta]
pub fn export_session_data(
    service: State<'_, CalibrationService>,
    session_id: String,
    format: String,
) -> Result<String, String> {
    let detail = service.get_session_detail(&session_id)?;
    let detail = detail.ok_or_else(|| "Session not found".to_string())?;

    let temp_path = std::env::temp_dir().join(format!(
        "artifexprocal_{}.{}", session_id, format.to_lowercase()));

    let mut file = std::fs::File::create(&temp_path).map_err(|e| e.to_string())?;

    match format.to_lowercase().as_str() {
        "csv" => calibration_storage::export::SessionExporter::export_csv(&detail, &mut file)
            .map_err(|e| e.to_string())?,
        "json" => calibration_storage::export::SessionExporter::export_json(&detail, &mut file)
            .map_err(|e| e.to_string())?,
        _ => return Err(format!("Unsupported format: {}", format)),
    }

    Ok(temp_path.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn generate_report(
    service: State<'_, CalibrationService>,
    request: crate::ipc::models::ReportRequestDto,
) -> Result<crate::ipc::models::ReportResponseDto, String> {
    let detail = service
        .get_session_detail(&request.session_id)?
        .ok_or_else(|| "Session not found".to_string())?;

    let compare = if let Some(ref id) = request.compare_session_id {
        Some(
            service
                .get_session_detail(id)?
                .ok_or_else(|| "Comparison session not found".to_string())?,
        )
    } else {
        None
    };

    let bytes = reporting::ReportEngine::generate(
        request.template,
        request.format,
        &detail,
        compare.as_ref(),
    )
    .map_err(|e| e.to_string())?;

    let ext = match request.format {
        reporting::types::ReportFormat::Html => "html",
        reporting::types::ReportFormat::Pdf => "pdf",
    };

    let temp_path = std::env::temp_dir().join(format!(
        "artifexprocal_report_{}.{}_{}",
        request.session_id,
        request.template.to_string().to_lowercase().replace(" ", "_").replace("/", "_"),
        ext
    ));

    std::fs::write(&temp_path, bytes).map_err(|e| e.to_string())?;

    Ok(crate::ipc::models::ReportResponseDto {
        path: temp_path.to_string_lossy().to_string(),
        format: request.format,
    })
}
