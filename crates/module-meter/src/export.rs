//! Export serialization for MeterModule measurement history.
//!
//! Phase 1 supports JSON (schema-validated) and CSV (RFC 4180) export formats.

use color_science::measurement::MeasurementResult;
use color_science::types::RgbSpace;
use serde::Serialize;
use std::collections::VecDeque;

/// In-memory measurement history: FIFO capped at 1000 entries.
pub type MeasurementHistory = VecDeque<MeasurementResult>;

/// Capacity of the in-memory history buffer.
pub const HISTORY_CAPACITY: usize = 1000;

/// Push a measurement into history, evicting oldest if at capacity.
pub fn push_history(history: &mut MeasurementHistory, m: MeasurementResult) {
    if history.len() >= HISTORY_CAPACITY {
        history.pop_front();
    }
    history.push_back(m);
}

// ---------------------------------------------------------------------------
// JSON export (matches docs/schemas/meter-export-phase1.json)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Phase1Export {
    measurement_uuid: String,
    schema_version: String,
    software_version: String,
    timestamp: String,
    mode: String,
    instrument: InstrumentExport,
    xyz: XyzExport,
    xyy: XyyExport,
    lab: LabExport,
    lch: LchExport,
    uv_prime: UvPrimeExport,
    cct: f64,
    duv: f64,
    delta_e2000: Option<f64>,
    target: Option<TargetExport>,
    patch_rgb: Option<RgbExport>,
    patch_bit_depth: Option<u8>,
    patch_colorspace: String,
    reference_white: String,
    session_id: Option<String>,
    sequence_index: Option<usize>,
    label: String,
}

#[derive(Serialize)]
struct InstrumentExport {
    model: String,
    id: String,
}

#[derive(Serialize)]
struct XyzExport {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize)]
struct XyyExport {
    x: f64,
    y: f64,
    #[serde(rename = "yLum")]
    y_lum: f64,
}

#[derive(Serialize)]
struct LabExport {
    l: f64,
    a: f64,
    b: f64,
}

#[derive(Serialize)]
struct LchExport {
    l: f64,
    c: f64,
    h: f64,
}

#[derive(Serialize)]
struct UvPrimeExport {
    u: f64,
    v: f64,
}

#[derive(Serialize)]
struct TargetExport {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
struct RgbExport {
    r: u16,
    g: u16,
    b: u16,
}

fn rgb_space_to_string(space: Option<RgbSpace>) -> String {
    match space {
        Some(RgbSpace::Rec709) => "BT.709".to_string(),
        Some(RgbSpace::Rec2020) => "BT.2020".to_string(),
        Some(RgbSpace::DciP3) => "DCI-P3".to_string(),
        Some(RgbSpace::DisplayP3) => "Display-P3".to_string(),
        Some(RgbSpace::Srgb) => "sRGB".to_string(),
        Some(RgbSpace::AdobeRgb) => "AdobeRGB".to_string(),
        Some(RgbSpace::ProPhoto) => "ProPhoto".to_string(),
        None => "".to_string(),
    }
}

fn measurement_to_export(m: &MeasurementResult) -> Phase1Export {
    Phase1Export {
        measurement_uuid: m.measurement_uuid.to_string(),
        schema_version: m.schema_version.clone(),
        software_version: m.software_version.clone(),
        timestamp: m.timestamp.clone(),
        mode: "Emissive".to_string(),
        instrument: InstrumentExport {
            model: m.instrument_model.clone(),
            id: m.instrument_id.clone(),
        },
        xyz: XyzExport {
            x: m.xyz.x,
            y: m.xyz.y,
            z: m.xyz.z,
        },
        xyy: XyyExport {
            x: m.xyy.x,
            y: m.xyy.y,
            y_lum: m.xyy.y_lum,
        },
        lab: LabExport {
            l: m.lab.l,
            a: m.lab.a,
            b: m.lab.b,
        },
        lch: LchExport {
            l: m.lch.l,
            c: m.lch.c,
            h: m.lch.h,
        },
        uv_prime: UvPrimeExport {
            u: m.uv_prime.u,
            v: m.uv_prime.v,
        },
        cct: m.cct.unwrap_or(0.0),
        duv: m.duv.unwrap_or(0.0),
        delta_e2000: m.delta_e_2000,
        target: m.target_xy.map(|(x, y)| TargetExport { x, y }),
        patch_rgb: m.patch_colorspace.map(|_| RgbExport {
            r: m.patch_rgb.r,
            g: m.patch_rgb.g,
            b: m.patch_rgb.b,
        }),
        patch_bit_depth: m.patch_colorspace.map(|_| m.patch_bit_depth),
        patch_colorspace: rgb_space_to_string(m.patch_colorspace),
        reference_white: m.reference_white.clone(),
        session_id: m.session_id.clone(),
        sequence_index: m.sequence_index,
        label: m.label.clone().unwrap_or_default(),
    }
}

/// Serialize a slice of measurements to a JSON array string.
pub fn export_json(measurements: &[MeasurementResult]) -> Result<String, serde_json::Error> {
    let exports: Vec<_> = measurements.iter().map(measurement_to_export).collect();
    serde_json::to_string_pretty(&exports)
}

// ---------------------------------------------------------------------------
// CSV export (RFC 4180, 35 columns)
// ---------------------------------------------------------------------------

use csv::WriterBuilder;
use std::io::Cursor;

/// CSV header row in exact column order per meter-module.md §3.1.1.
const CSV_HEADER: &[&str] = &[
    "measurement_uuid",
    "schema_version",
    "software_version",
    "timestamp",
    "mode",
    "instrument_model",
    "instrument_id",
    "x",
    "y",
    "z",
    "xy_x",
    "xy_y",
    "lab_l",
    "lab_a",
    "lab_b",
    "lch_l",
    "lch_c",
    "lch_h",
    "uvp_u",
    "uvp_v",
    "cct",
    "duv",
    "delta_e_2000",
    "target_x",
    "target_y",
    "patch_r",
    "patch_g",
    "patch_b",
    "patch_bit_depth",
    "patch_colorspace",
    "reference_white",
    "session_id",
    "sequence_index",
    "label",
];

/// Serialize a slice of measurements to a CSV string.
pub fn export_csv(measurements: &[MeasurementResult]) -> Result<String, csv::Error> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut wtr = WriterBuilder::new().from_writer(&mut cursor);
        wtr.write_record(CSV_HEADER)?;
        for m in measurements {
            wtr.write_record(csv_row(m))?;
        }
        wtr.flush()?;
    }
    let bytes = cursor.into_inner();
    Ok(String::from_utf8(bytes).expect("CSV writer produces valid UTF-8"))
}

fn csv_row(m: &MeasurementResult) -> Vec<String> {
    vec![
        m.measurement_uuid.to_string(),
        m.schema_version.clone(),
        m.software_version.clone(),
        m.timestamp.clone(),
        "Emissive".to_string(),
        m.instrument_model.clone(),
        m.instrument_id.clone(),
        fmt_f64(m.xyz.x),
        fmt_f64(m.xyz.y),
        fmt_f64(m.xyz.z),
        fmt_f64(m.xyy.x),
        fmt_f64(m.xyy.y),
        fmt_f64(m.lab.l),
        fmt_f64(m.lab.a),
        fmt_f64(m.lab.b),
        fmt_f64(m.lch.l),
        fmt_f64(m.lch.c),
        fmt_f64(m.lch.h),
        fmt_f64(m.uv_prime.u),
        fmt_f64(m.uv_prime.v),
        fmt_opt_f64(m.cct),
        fmt_opt_f64(m.duv),
        fmt_opt_f64(m.delta_e_2000),
        fmt_opt_f64(m.target_xy.map(|t| t.0)),
        fmt_opt_f64(m.target_xy.map(|t| t.1)),
        m.patch_colorspace.map(|_| m.patch_rgb.r.to_string()).unwrap_or_default(),
        m.patch_colorspace.map(|_| m.patch_rgb.g.to_string()).unwrap_or_default(),
        m.patch_colorspace.map(|_| m.patch_rgb.b.to_string()).unwrap_or_default(),
        m.patch_colorspace.map(|_| m.patch_bit_depth.to_string()).unwrap_or_default(),
        rgb_space_to_string(m.patch_colorspace),
        m.reference_white.clone(),
        m.session_id.clone().unwrap_or_default(),
        m.sequence_index.map(|n| n.to_string()).unwrap_or_default(),
        m.label.clone().unwrap_or_default(),
    ]
}

fn fmt_f64(v: f64) -> String {
    // Use enough precision to round-trip f64 without trailing zeros.
    format!("{:.6}", v).trim_end_matches('0').trim_end_matches('.').to_string()
}

fn fmt_opt_f64(v: Option<f64>) -> String {
    match v {
        Some(f) => fmt_f64(f),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Schema validation
// ---------------------------------------------------------------------------

/// Validate a JSON export string against the Phase 1 schema.
/// Returns Ok(()) if valid, or an error string describing the first failure.
pub fn validate_export_json(json_str: &str) -> Result<(), String> {
    static SCHEMA_JSON: &str = include_str!("../../../docs/schemas/meter-export-phase1.json");
    let schema: serde_json::Value =
        serde_json::from_str(SCHEMA_JSON).map_err(|e| format!("schema parse: {}", e))?;
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("json parse: {}", e))?;

    // Export is an array of objects; validate each element against the schema.
    let items = value.as_array().ok_or_else(|| "export json must be an array".to_string())?;
    for (i, item) in items.iter().enumerate() {
        jsonschema::validate(&schema, item)
            .map_err(|e| format!("item {}: {}", i, e))?;
    }
    Ok(())
}
