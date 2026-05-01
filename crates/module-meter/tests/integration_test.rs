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

#[tokio::test]
async fn read_emits_measurement_received() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let mut rx = event_bus.subscribe();

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({ "instrument_id": "fake-meter-1" }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command("read", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("should receive event")
        .expect("channel should be open");

    assert!(
        matches!(event, app_core::ModuleEvent::MeasurementReceived { .. }),
        "expected MeasurementReceived, got {:?}",
        event
    );
}

#[tokio::test]
async fn read_continuous_sequence_success() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let mut rx = event_bus.subscribe();

    let xyz1 = color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 };
    let xyz2 = color_science::types::Xyz { x: 11.0, y: 21.0, z: 31.0 };
    let xyz3 = color_science::types::Xyz { x: 12.0, y: 22.0, z: 32.0 };

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "Sequence": {
                        "values": [
                            { "Ok": xyz1 },
                            { "Ok": xyz2 },
                            { "Ok": xyz3 }
                        ],
                        "loop_at_end": false
                    }
                }
            }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command(
            "read_continuous",
            serde_json::json!({ "meter_id": meter_id, "interval_ms": 10 }),
        )
        .unwrap();

    for i in 0..3 {
        let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
            .await
            .expect(&format!("should receive measurement {}", i))
            .expect("channel open");
        assert!(
            matches!(event, app_core::ModuleEvent::MeasurementReceived { .. }),
            "expected MeasurementReceived at index {}, got {:?}",
            i,
            event
        );
    }

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("should receive stopped event")
        .expect("channel open");
    assert!(
        matches!(
            event,
            app_core::ModuleEvent::ContinuousReadStopped {
                reason: app_core::ContinuousReadStopReason::SequenceExhausted,
                ..
            }
        ),
        "expected SequenceExhausted, got {:?}",
        event
    );
}

#[tokio::test]
async fn read_continuous_three_error_tolerance() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let mut rx = event_bus.subscribe();

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "Sequence": {
                        "values": [
                            { "Err": { "Timeout": null } },
                            { "Err": { "Timeout": null } },
                            { "Err": { "Timeout": null } },
                            { "Err": { "Timeout": null } }
                        ],
                        "loop_at_end": false
                    }
                }
            }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command(
            "read_continuous",
            serde_json::json!({ "meter_id": meter_id, "interval_ms": 10 }),
        )
        .unwrap();

    for expected_count in 1..=3 {
        let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
            .await
            .expect(&format!("should receive error {}", expected_count))
            .expect("channel open");
        assert!(
            matches!(
                event,
                app_core::ModuleEvent::ContinuousReadError { consecutive_count, .. }
                if consecutive_count == expected_count
            ),
            "expected ContinuousReadError with count {}, got {:?}",
            expected_count,
            event
        );
    }

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("should receive stopped event")
        .expect("channel open");
    assert!(
        matches!(
            event,
            app_core::ModuleEvent::ContinuousReadStopped {
                reason: app_core::ContinuousReadStopReason::ErrorToleranceExceeded,
                ..
            }
        ),
        "expected ErrorToleranceExceeded, got {:?}",
        event
    );
}

#[tokio::test]
async fn read_continuous_mixed_errors() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let mut rx = event_bus.subscribe();

    let xyz = color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 };

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "Sequence": {
                        "values": [
                            { "Ok": xyz },
                            { "Err": { "Timeout": null } },
                            { "Ok": xyz },
                            { "Err": { "Timeout": null } },
                            { "Ok": xyz }
                        ],
                        "loop_at_end": false
                    }
                }
            }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command(
            "read_continuous",
            serde_json::json!({ "meter_id": meter_id, "interval_ms": 10 }),
        )
        .unwrap();

    let e1 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(e1, app_core::ModuleEvent::MeasurementReceived { .. }),
        "expected MeasurementReceived, got {:?}",
        e1
    );

    let e2 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(
            e2,
            app_core::ModuleEvent::ContinuousReadError { consecutive_count: 1, .. }
        ),
        "expected ContinuousReadError count=1, got {:?}",
        e2
    );

    let e3 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(e3, app_core::ModuleEvent::MeasurementReceived { .. }),
        "expected MeasurementReceived, got {:?}",
        e3
    );

    let e4 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(
            e4,
            app_core::ModuleEvent::ContinuousReadError { consecutive_count: 1, .. }
        ),
        "expected ContinuousReadError count=1, got {:?}",
        e4
    );

    let e5 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(e5, app_core::ModuleEvent::MeasurementReceived { .. }),
        "expected MeasurementReceived, got {:?}",
        e5
    );

    let e6 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(
            e6,
            app_core::ModuleEvent::ContinuousReadStopped {
                reason: app_core::ContinuousReadStopReason::SequenceExhausted,
                ..
            }
        ),
        "expected SequenceExhausted, got {:?}",
        e6
    );
}

#[tokio::test]
async fn stop_continuous_mid_sequence() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let mut rx = event_bus.subscribe();

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "PlanckianSweep": {
                        "start_cct": 3000.0,
                        "end_cct": 4000.0,
                        "steps": 100,
                        "target_luminance": 100.0,
                        "loop_at_end": true
                    }
                }
            }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command(
            "read_continuous",
            serde_json::json!({ "meter_id": meter_id, "interval_ms": 10 }),
        )
        .unwrap();

    let e1 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(e1, app_core::ModuleEvent::MeasurementReceived { .. }),
        "expected MeasurementReceived, got {:?}",
        e1
    );

    module
        .handle_command(
            "stop_continuous",
            serde_json::json!({ "meter_id": meter_id }),
        )
        .unwrap();

    let e2 = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(
        matches!(
            e2,
            app_core::ModuleEvent::ContinuousReadStopped {
                reason: app_core::ContinuousReadStopReason::Cancelled,
                ..
            }
        ),
        "expected Cancelled, got {:?}",
        e2
    );

    let maybe_event = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    assert!(
        maybe_event.is_err(),
        "expected no further events after stop, got {:?}",
        maybe_event
    );
}

#[tokio::test]
async fn single_read_rejected_during_continuous() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "PlanckianSweep": {
                        "start_cct": 3000.0,
                        "end_cct": 4000.0,
                        "steps": 100,
                        "target_luminance": 100.0,
                        "loop_at_end": true
                    }
                }
            }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command(
            "read_continuous",
            serde_json::json!({ "meter_id": meter_id, "interval_ms": 10 }),
        )
        .unwrap();

    let err = module
        .handle_command("read", serde_json::json!({ "meter_id": meter_id }))
        .expect_err("single read should be rejected during continuous read");

    let msg = err.to_string();
    assert!(
        msg.contains("rejected") || msg.contains("continuous"),
        "expected rejection message, got: {}",
        msg
    );

    module
        .handle_command(
            "stop_continuous",
            serde_json::json!({ "meter_id": meter_id }),
        )
        .unwrap();
}

// ── Register slot tests ────────────────────────────────────────────

#[tokio::test]
async fn set_current_get_all() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let measurement = color_science::measurement::MeasurementResult::from_xyz(
        color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 },
        "meter-1",
        "fake-1",
        "FakeMeter",
    );

    module
        .handle_command(
            "set_register",
            serde_json::json!({
                "slot": "Current",
                "measurement": measurement
            }),
        )
        .unwrap();

    let all = module
        .handle_command("get_all_registers", serde_json::json!({}))
        .unwrap();

    let map: std::collections::HashMap<app_core::RegisterSlot, color_science::measurement::MeasurementResult> =
        serde_json::from_value(all).unwrap();

    assert_eq!(map.len(), 1);
    assert!(map.contains_key(&app_core::RegisterSlot::Current));
    assert!(!map.contains_key(&app_core::RegisterSlot::Reference));
    assert!(!map.contains_key(&app_core::RegisterSlot::W));
}

#[tokio::test]
async fn set_all_three_slots() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    for (slot, xyz) in [
        ("Current", color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 }),
        ("Reference", color_science::types::Xyz { x: 11.0, y: 21.0, z: 31.0 }),
        ("W", color_science::types::Xyz { x: 12.0, y: 22.0, z: 32.0 }),
    ] {
        let measurement = color_science::measurement::MeasurementResult::from_xyz(
            xyz, "meter-1", "fake-1", "FakeMeter",
        );
        module
            .handle_command(
                "set_register",
                serde_json::json!({ "slot": slot, "measurement": measurement }),
            )
            .unwrap();
    }

    let all = module
        .handle_command("get_all_registers", serde_json::json!({}))
        .unwrap();
    let map: std::collections::HashMap<app_core::RegisterSlot, color_science::measurement::MeasurementResult> =
        serde_json::from_value(all).unwrap();

    assert_eq!(map.len(), 3);
    assert!(map.contains_key(&app_core::RegisterSlot::Current));
    assert!(map.contains_key(&app_core::RegisterSlot::Reference));
    assert!(map.contains_key(&app_core::RegisterSlot::W));
}

#[tokio::test]
async fn clear_reference() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    for (slot, xyz) in [
        ("Current", color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 }),
        ("Reference", color_science::types::Xyz { x: 11.0, y: 21.0, z: 31.0 }),
        ("W", color_science::types::Xyz { x: 12.0, y: 22.0, z: 32.0 }),
    ] {
        let measurement = color_science::measurement::MeasurementResult::from_xyz(
            xyz, "meter-1", "fake-1", "FakeMeter",
        );
        module
            .handle_command(
                "set_register",
                serde_json::json!({ "slot": slot, "measurement": measurement }),
            )
            .unwrap();
    }

    module
        .handle_command(
            "clear_register",
            serde_json::json!({ "slot": "Reference" }),
        )
        .unwrap();

    let all = module
        .handle_command("get_all_registers", serde_json::json!({}))
        .unwrap();
    let map: std::collections::HashMap<app_core::RegisterSlot, color_science::measurement::MeasurementResult> =
        serde_json::from_value(all).unwrap();

    assert_eq!(map.len(), 2);
    assert!(map.contains_key(&app_core::RegisterSlot::Current));
    assert!(!map.contains_key(&app_core::RegisterSlot::Reference));
    assert!(map.contains_key(&app_core::RegisterSlot::W));
}

#[tokio::test]
async fn set_current_twice() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let m1 = color_science::measurement::MeasurementResult::from_xyz(
        color_science::types::Xyz { x: 1.0, y: 2.0, z: 3.0 },
        "meter-1", "fake-1", "FakeMeter",
    );
    let m2 = color_science::measurement::MeasurementResult::from_xyz(
        color_science::types::Xyz { x: 4.0, y: 5.0, z: 6.0 },
        "meter-1", "fake-1", "FakeMeter",
    );

    module
        .handle_command(
            "set_register",
            serde_json::json!({ "slot": "Current", "measurement": m1 }),
        )
        .unwrap();
    module
        .handle_command(
            "set_register",
            serde_json::json!({ "slot": "Current", "measurement": m2.clone() }),
        )
        .unwrap();

    let all = module
        .handle_command("get_all_registers", serde_json::json!({}))
        .unwrap();
    let map: std::collections::HashMap<app_core::RegisterSlot, color_science::measurement::MeasurementResult> =
        serde_json::from_value(all).unwrap();

    assert_eq!(map.len(), 1);
    let stored = map.get(&app_core::RegisterSlot::Current).unwrap();
    assert!(
        (stored.xyz.x - m2.xyz.x).abs() < 0.001,
        "expected second measurement (last-write-wins)"
    );
}

#[tokio::test]
async fn invalid_slot_name() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let measurement = color_science::measurement::MeasurementResult::from_xyz(
        color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 },
        "meter-1", "fake-1", "FakeMeter",
    );

    let err = module
        .handle_command(
            "set_register",
            serde_json::json!({ "slot": "Bogus", "measurement": measurement }),
        )
        .expect_err("Bogus slot should fail");

    let msg = err.to_string();
    assert!(
        msg.contains("unknown variant")
            && msg.contains("Bogus")
            && msg.contains("Current")
            && msg.contains("Reference")
            && msg.contains("W"),
        "expected human-readable unknown variant error, got: {}",
        msg
    );
}

#[tokio::test]
async fn read_populates_current() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let connect = module
        .handle_command("connect", serde_json::json!({ "instrument_id": "fake-meter-1" }))
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command("read", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();

    let all = module
        .handle_command("get_all_registers", serde_json::json!({}))
        .unwrap();
    let map: std::collections::HashMap<app_core::RegisterSlot, color_science::measurement::MeasurementResult> =
        serde_json::from_value(all).unwrap();

    assert_eq!(map.len(), 1);
    assert!(map.contains_key(&app_core::RegisterSlot::Current));
}

#[tokio::test]
async fn event_on_set() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let mut rx = event_bus.subscribe();

    let measurement = color_science::measurement::MeasurementResult::from_xyz(
        color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 },
        "meter-1", "fake-1", "FakeMeter",
    );

    module
        .handle_command(
            "set_register",
            serde_json::json!({ "slot": "Reference", "measurement": measurement.clone() }),
        )
        .unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("should receive event")
        .expect("channel open");

    assert!(
        matches!(
            event,
            app_core::ModuleEvent::RegisterChanged {
                slot: app_core::RegisterSlot::Reference,
                measurement: Some(_),
            }
        ),
        "expected RegisterChanged(Reference, Some), got {:?}",
        event
    );
}

#[tokio::test]
async fn event_on_clear() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let measurement = color_science::measurement::MeasurementResult::from_xyz(
        color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 },
        "meter-1", "fake-1", "FakeMeter",
    );

    module
        .handle_command(
            "set_register",
            serde_json::json!({ "slot": "W", "measurement": measurement }),
        )
        .unwrap();

    let mut rx = event_bus.subscribe();

    module
        .handle_command("clear_register", serde_json::json!({ "slot": "W" }))
        .unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("should receive event")
        .expect("channel open");

    assert!(
        matches!(
            event,
            app_core::ModuleEvent::RegisterChanged {
                slot: app_core::RegisterSlot::W,
                measurement: None,
            }
        ),
        "expected RegisterChanged(W, None), got {:?}",
        event
    );
}

#[tokio::test]
async fn no_register_changed_on_read() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let mut rx = event_bus.subscribe();

    let connect = module
        .handle_command("connect", serde_json::json!({ "instrument_id": "fake-meter-1" }))
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    module
        .handle_command("read", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();

    // Expect MeasurementReceived — but NOT RegisterChanged.
    let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("should receive event")
        .expect("channel open");

    assert!(
        matches!(event, app_core::ModuleEvent::MeasurementReceived { .. }),
        "expected MeasurementReceived, got {:?}",
        event
    );

    // Verify no further events (specifically no RegisterChanged).
    let maybe = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    assert!(
        maybe.is_err(),
        "expected no RegisterChanged on auto-populated Current, got {:?}",
        maybe
    );
}
