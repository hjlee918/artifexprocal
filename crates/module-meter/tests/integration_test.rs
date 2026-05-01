//! Integration test: MeterModule end-to-end with FakeMeter.
//!
//! Proves that MeterModule can:
//! 1. Detect instruments (FakeMeter in Phase 1)
//! 2. Connect to an instrument
//! 3. Take a measurement via the "read" command
//! 4. Produce a valid MeasurementResult

use app_core::{CalibrationModule, EventBus, ModuleContext};
use color_science::types::Xyz;
use std::sync::Arc;

#[tokio::test]
async fn meter_module_fake_meter_end_to_end() {
    // ── Setup ──────────────────────────────────────────────
    let event_bus = Arc::new(EventBus::new());
    let ctx = ModuleContext::new(event_bus.clone());

    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    // ── Step 1: Detect ─────────────────────────────────────
    let detected = module
        .handle_command("detect", serde_json::json!({}))
        .expect("detect should succeed");
    let instruments: Vec<serde_json::Value> =
        serde_json::from_value(detected).expect("detect should return array");
    assert!(
        !instruments.is_empty(),
        "detect should find at least one instrument"
    );
    let fake = &instruments[0];
    assert_eq!(
        fake["model"].as_str(),
        Some("FakeMeter"),
        "first instrument should be FakeMeter"
    );

    // ── Step 2: Connect ──────────────────────────────────
    let instrument_id = fake["id"].as_str().unwrap();
    let connect_result = module
        .handle_command(
            "connect",
            serde_json::json!({ "instrument_id": instrument_id }),
        )
        .expect("connect should succeed");
    let meter_id = connect_result["meter_id"]
        .as_str()
        .expect("connect should return meter_id");

    // ── Step 3: Read ─────────────────────────────────────
    let read_result = module
        .handle_command(
            "read",
            serde_json::json!({ "meter_id": meter_id }),
        )
        .expect("read should succeed");

    // ── Step 4: Verify MeasurementResult ───────────────────
    let result: color_science::measurement::MeasurementResult =
        serde_json::from_value(read_result).expect("read should return MeasurementResult");

    // D65 reference values
    let d65 = Xyz {
        x: 95.047,
        y: 100.0,
        z: 108.883,
    };
    assert!(
        (result.xyz.x - d65.x).abs() < 0.1,
        "X should be ~95.047, got {}",
        result.xyz.x
    );
    assert!(
        (result.xyz.y - d65.y).abs() < 0.1,
        "Y should be ~100.0, got {}",
        result.xyz.y
    );
    assert!(
        (result.xyz.z - d65.z).abs() < 0.1,
        "Z should be ~108.883, got {}",
        result.xyz.z
    );

    assert_eq!(result.instrument_model, "FakeMeter");
    assert_eq!(result.schema_version, "1.0");
    assert_eq!(result.reference_white, "D65");

    // ── Step 5: Disconnect ───────────────────────────────
    module
        .handle_command(
            "disconnect",
            serde_json::json!({ "meter_id": meter_id }),
        )
        .expect("disconnect should succeed");
}

#[tokio::test]
async fn meter_module_planckian_sweep() {
    // ── Setup ──────────────────────────────────────────────
    let event_bus = Arc::new(EventBus::new());
    let ctx = ModuleContext::new(event_bus.clone());

    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    // ── Connect with PlanckianSweep config ─────────────────
    let connect_result = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "PlanckianSweep": {
                        "start_cct": 3000.0,
                        "end_cct": 10000.0,
                        "steps": 8,
                        "target_luminance": 100.0,
                        "loop_at_end": false
                    }
                }
            }),
        )
        .expect("connect with PlanckianSweep should succeed");
    let meter_id = connect_result["meter_id"]
        .as_str()
        .expect("connect should return meter_id");

    // ── Read 8 times, collect XYZ ────────────────────────
    let mut readings = Vec::new();
    for _ in 0..8 {
        let read_result = module
            .handle_command(
                "read",
                serde_json::json!({ "meter_id": meter_id }),
            )
            .expect("read should succeed");
        let result: color_science::measurement::MeasurementResult =
            serde_json::from_value(read_result).expect("read should return MeasurementResult");
        readings.push(result.xyz);
    }

    // All 8 readings must be pairwise distinct.
    for i in 0..readings.len() {
        for j in (i + 1)..readings.len() {
            assert_ne!(
                readings[i], readings[j],
                "readings at index {} and {} should differ",
                i, j
            );
        }
    }

    // ── Read past end should fail ─────────────────────────
    let err = module
        .handle_command(
            "read",
            serde_json::json!({ "meter_id": meter_id }),
        )
        .expect_err("read past end should fail");
    assert!(
        err.to_string().contains("SequenceExhausted")
            || err.to_string().contains("sequence exhausted"),
        "expected SequenceExhausted, got: {}",
        err
    );

    // ── Disconnect ─────────────────────────────────────────
    module
        .handle_command(
            "disconnect",
            serde_json::json!({ "meter_id": meter_id }),
        )
        .expect("disconnect should succeed");
}
