use calibration_core::state::{CalibrationTier, SessionConfig, TargetSpace, ToneCurve, WhitePoint};
use calibration_storage::query::{ComputedResults, PatchReading, SessionDetail, SessionSummary};
use color_science::types::XYZ;
use reporting::{ReportEngine, ReportFormat, ReportTemplate};

fn mock_session_detail(name: &str) -> SessionDetail {
    SessionDetail {
        summary: SessionSummary {
            id: "test-1".to_string(),
            name: name.to_string(),
            created_at: 1714219200,
            ended_at: Some(1714222800),
            state: "finished".to_string(),
            target_space: "Rec.709".to_string(),
            tier: Some("Pro".to_string()),
            patch_count: 10,
            gamma: Some(2.4),
            max_de: Some(1.23),
            avg_de: Some(0.56),
        },
        config: SessionConfig {
            name: name.to_string(),
            target_space: TargetSpace::Bt709,
            tone_curve: ToneCurve::Gamma(2.4),
            white_point: WhitePoint::D65,
            patch_count: 10,
            reads_per_patch: 3,
            settle_time_ms: 5000,
            stability_threshold: None,
            tier: CalibrationTier::GrayscaleOnly,
            manual_patches: None,
        },
        readings: vec![
            PatchReading {
                patch_index: 0,
                target_rgb: (1.0, 1.0, 1.0),
                measured_xyz: XYZ { x: 95.0, y: 100.0, z: 108.0 },
                reading_index: 0,
                measurement_type: "cal".to_string(),
            },
        ],
        results: Some(ComputedResults {
            gamma: Some(2.4),
            max_de: Some(1.23),
            avg_de: Some(0.56),
            white_balance: Some("D65".to_string()),
            lut_1d_size: Some(1024),
            lut_3d_size: None,
        }),
    }
}

#[test]
fn test_engine_generate_html() {
    let detail = mock_session_detail("Integration Test");
    let bytes = ReportEngine::generate(
        ReportTemplate::QuickSummary,
        ReportFormat::Html,
        &detail,
        None,
    )
    .unwrap();
    let html = String::from_utf8(bytes).unwrap();
    assert!(html.contains("Integration Test"));
    assert!(html.contains("<!DOCTYPE html>"));
}

#[test]
fn test_engine_generate_pdf() {
    let detail = mock_session_detail("Integration Test");
    let bytes = ReportEngine::generate(
        ReportTemplate::QuickSummary,
        ReportFormat::Pdf,
        &detail,
        None,
    )
    .unwrap();
    assert_eq!(&bytes[0..5], b"%PDF-");
}

#[test]
fn test_pre_post_comparison_requires_compare() {
    let detail = mock_session_detail("Single");
    let result = ReportEngine::generate(
        ReportTemplate::PrePostComparison,
        ReportFormat::Html,
        &detail,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_pre_post_comparison_success() {
    let before = mock_session_detail("Before");
    let after = mock_session_detail("After");
    let bytes = ReportEngine::generate(
        ReportTemplate::PrePostComparison,
        ReportFormat::Html,
        &after,
        Some(&before),
    )
    .unwrap();
    let html = String::from_utf8(bytes).unwrap();
    assert!(html.contains("Before"));
    assert!(html.contains("After"));
}
