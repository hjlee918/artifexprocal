//! MeasurementResult — the universal colorimetric data contract.
//!
//! Lives in the `color-science` crate (per v2-architecture.md §4.1
//! and meter-module.md §2.11). Consumed by workflow engine, storage,
//! visualization, and reporting modules.

use crate::types::{ICtCp, Lab, LCh, Rgb, RgbSpace, UvPrime, XyY, Xyz};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single colorimetric measurement from any supported instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementResult {
    /// UUID v4 assigned at measurement time; stable across re-exports.
    pub measurement_uuid: Uuid,

    /// UTC timestamp when the measurement completed.
    pub timestamp: String, // ISO 8601 with ms + Z

    /// Phase 1 schema version: "1.0"
    pub schema_version: String,

    /// Application version string.
    pub software_version: String,

    /// Identifier of the meter module that produced this reading.
    pub meter_id: String,

    /// Identifier of the specific instrument (serial number or USB path).
    pub instrument_id: String,

    /// Name of the instrument model (e.g., "i1 Display Pro Rev.B").
    pub instrument_model: String,

    // --- Raw instrument data ---
    /// CIE XYZ tristimulus values (cd/m² for Y).
    pub xyz: Xyz,

    /// Derived CIE xyY chromaticity + luminance.
    pub xyy: XyY,

    /// Derived CIE Lab (D65 reference white).
    pub lab: Lab,

    /// Derived CIE LCh (Lightness, Chroma, Hue).
    pub lch: LCh,

    /// Derived CIE 1976 u′v′ (UCS) chromaticity.
    pub uv_prime: UvPrime,

    /// ICtCp perceptual color difference space (for HDR).
    pub ictcp: Option<ICtCp>,

    // --- Color Temperature & Quality ---
    /// Correlated Color Temperature (Kelvin).
    pub cct: Option<f64>,

    /// Distance from Planckian locus.
    /// Positive = green side (higher v′); negative = magenta side, per Ohno 2013.
    pub duv: Option<f64>,

    // --- Target and error metrics ---
    /// Target color (the intended RGB or xyY of the displayed patch).
    pub target_xy: Option<(f64, f64)>,

    /// DeltaE 2000 against target (if target is known).
    pub delta_e_2000: Option<f64>,

    /// DeltaE 1976 (euclidean in Lab).
    pub delta_e_76: Option<f64>,

    // --- Instrument metadata ---
    /// Integration time in milliseconds.
    pub integration_time_ms: Option<u32>,

    /// Whether the instrument reported the reading as saturated.
    pub saturated: bool,

    // --- Patch context ---
    /// The RGB stimulus that was displayed (16-bit full range, 0–65535).
    pub patch_rgb: Rgb<u16>,

    /// Source bit depth of the patch (8, 10, 12, or 16).
    pub patch_bit_depth: u8,

    /// Colorspace tag for the RGB stimulus.
    pub patch_colorspace: Option<RgbSpace>,

    /// Reference white point: "D65" in Phase 1.
    pub reference_white: String,

    /// Which pattern generator produced the patch.
    pub pattern_generator_id: Option<String>,

    /// Which display was being measured.
    pub display_id: Option<String>,

    /// Picture mode active on the display during measurement.
    pub picture_mode: Option<String>,

    // --- Session context ---
    /// Workflow session this reading belongs to.
    pub session_id: Option<String>,

    /// Sequential index within the session.
    pub sequence_index: Option<usize>,

    /// User-defined label or tag.
    pub label: Option<String>,
}

impl MeasurementResult {
    /// Create a MeasurementResult from raw XYZ with default/derived fields populated.
    pub fn from_xyz(
        xyz: Xyz,
        meter_id: impl Into<String>,
        instrument_id: impl Into<String>,
        instrument_model: impl Into<String>,
    ) -> Self {
        let xyy: XyY = xyz.into();
        let lab: Lab = xyz.into();
        let lch: LCh = lab.into();
        let uv_prime = crate::conversion::xyz_to_uv_prime(xyz);
        let ictcp = Some(crate::conversion::xyz_to_ictcp(xyz));

        MeasurementResult {
            measurement_uuid: Uuid::new_v4(),
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            schema_version: "1.0".to_string(),
            software_version: env!("CARGO_PKG_VERSION").to_string(),
            meter_id: meter_id.into(),
            instrument_id: instrument_id.into(),
            instrument_model: instrument_model.into(),
            xyz,
            xyy,
            lab,
            lch,
            uv_prime,
            ictcp,
            cct: None,
            duv: None,
            target_xy: None,
            delta_e_2000: None,
            delta_e_76: None,
            integration_time_ms: None,
            saturated: false,
            patch_rgb: Rgb { r: 0, g: 0, b: 0 },
            patch_bit_depth: 8,
            patch_colorspace: None,
            reference_white: "D65".to_string(),
            pattern_generator_id: None,
            display_id: None,
            picture_mode: None,
            session_id: None,
            sequence_index: None,
            label: None,
        }
    }
}
