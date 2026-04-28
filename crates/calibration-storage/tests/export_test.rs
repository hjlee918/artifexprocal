use calibration_core::state::{CalibrationTier, SessionConfig, TargetSpace, ToneCurve, WhitePoint};
use calibration_storage::export::SessionExporter;
use calibration_storage::query::{SessionDetail, SessionSummary, ComputedResults, PatchReading};
use color_science::types::XYZ;

fn make_test_detail() -> SessionDetail {
    SessionDetail {
        summary: SessionSummary {
            id: "test-id".to_string(),
            name: "Test Session".to_string(),
            created_at: 1714320000000,
            ended_at: Some(1714323600000),
            state: "finished".to_string(),
            target_space: "BT.709".to_string(),
            tier: Some("Full3D".to_string()),
            patch_count: 2,
            gamma: Some(2.4),
            max_de: Some(1.23),
            avg_de: Some(0.45),
        },
        config: SessionConfig {
            name: "Test Session".to_string(),
            target_space: TargetSpace::Bt709,
            tone_curve: ToneCurve::Gamma(2.4),
            white_point: WhitePoint::D65,
            patch_count: 2,
            reads_per_patch: 1,
            settle_time_ms: 0,
            stability_threshold: None,
            tier: CalibrationTier::Full3D,
        },
        readings: vec![
            PatchReading {
                patch_index: 0,
                target_rgb: (0.0, 0.0, 0.0),
                measured_xyz: XYZ { x: 0.52, y: 0.55, z: 0.62 },
                reading_index: 0,
                measurement_type: "cal".to_string(),
            },
            PatchReading {
                patch_index: 1,
                target_rgb: (1.0, 1.0, 1.0),
                measured_xyz: XYZ { x: 95.0, y: 100.0, z: 108.0 },
                reading_index: 0,
                measurement_type: "cal".to_string(),
            },
        ],
        results: Some(ComputedResults {
            gamma: Some(2.4),
            max_de: Some(1.23),
            avg_de: Some(0.45),
            white_balance: Some("[0.0, 0.0, 0.0]".to_string()),
            lut_1d_size: Some(256),
            lut_3d_size: Some(33),
        }),
    }
}

#[test]
fn test_export_csv() {
    let detail = make_test_detail();
    let mut buf = Vec::new();
    SessionExporter::export_csv(&detail, &mut buf).unwrap();
    let csv = String::from_utf8(buf).unwrap();

    assert!(csv.starts_with("patch_index,target_r,target_g,target_b,measured_x,measured_y,measured_z"));
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 3); // header + 2 readings
    assert!(lines[1].contains("0.0000,0.0000,0.0000"));
    assert!(lines[2].contains("1.0000,1.0000,1.0000"));
}

#[test]
fn test_export_json() {
    let detail = make_test_detail();
    let mut buf = Vec::new();
    SessionExporter::export_json(&detail, &mut buf).unwrap();
    let json_str = String::from_utf8(buf).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["session_id"], "test-id");
    assert_eq!(parsed["name"], "Test Session");
    assert_eq!(parsed["state"], "finished");
    assert!(parsed["results"]["gamma"].as_f64().is_some());
    let readings = parsed["readings"].as_array().unwrap();
    assert_eq!(readings.len(), 2);
}
