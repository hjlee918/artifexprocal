use calibration_core::state::{CalibrationTier, SessionConfig, TargetSpace, ToneCurve, WhitePoint};
use calibration_storage::query::{ComputedResults, PatchReading, SessionDetail, SessionSummary};
use color_science::types::XYZ;
use reporting::template::{render_detailed, render_pre_post_comparison, render_quick_summary};

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
        },
        readings: vec![
            PatchReading {
                patch_index: 0,
                target_rgb: (1.0, 1.0, 1.0),
                measured_xyz: XYZ { x: 95.0, y: 100.0, z: 108.0 },
                reading_index: 0,
                measurement_type: "cal".to_string(),
            },
            PatchReading {
                patch_index: 1,
                target_rgb: (0.5, 0.5, 0.5),
                measured_xyz: XYZ { x: 48.0, y: 50.0, z: 54.0 },
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
fn test_quick_summary_contains_session_name() {
    let detail = mock_session_detail("LG OLED SDR");
    let html = render_quick_summary(&detail);
    assert!(html.contains("LG OLED SDR"));
    assert!(html.contains("Quick Summary"));
    assert!(html.contains("<svg"));
    assert!(html.contains("</svg>"));
}

#[test]
fn test_detailed_contains_readings_table() {
    let detail = mock_session_detail("Detailed Test");
    let html = render_detailed(&detail);
    assert!(html.contains("Detailed Calibration Report"));
    assert!(html.contains("Patch Readings"));
    assert!(html.contains("<table>"));
    assert!(html.contains("<svg"));
}

#[test]
fn test_pre_post_comparison_contains_both_sessions() {
    let before = mock_session_detail("Before Session");
    let after = mock_session_detail("After Session");
    let html = render_pre_post_comparison(&before, &after);
    assert!(html.contains("Before Session"));
    assert!(html.contains("After Session"));
    assert!(html.contains("Pre/Post Calibration Comparison"));
    assert!(html.contains("Delta Summary"));
}
