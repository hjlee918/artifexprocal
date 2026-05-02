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

// ── Export / History tests ───────────────────────────────────────

#[tokio::test]
async fn read_populates_history() {
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

    let json_export = module
        .handle_command("export_json", serde_json::json!({}))
        .unwrap();
    let json_str = json_export["json"].as_str().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["instrument"]["model"], "FakeMeter");

    let csv_export = module
        .handle_command("export_csv", serde_json::json!({}))
        .unwrap();
    let csv_str = csv_export["csv"].as_str().unwrap();
    let lines: Vec<&str> = csv_str.lines().collect();
    assert_eq!(lines.len(), 2, "expected header + 1 data row");
    assert!(lines[0].starts_with("measurement_uuid"));
}

#[tokio::test]
async fn clear_history_empties_export() {
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

    module
        .handle_command("clear_history", serde_json::json!({}))
        .unwrap();

    let json_export = module
        .handle_command("export_json", serde_json::json!({}))
        .unwrap();
    let json_str = json_export["json"].as_str().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
    assert!(arr.is_empty());

    // CSV export after clear must still contain the 35-column header row.
    let csv_export = module
        .handle_command("export_csv", serde_json::json!({}))
        .unwrap();
    let csv_str = csv_export["csv"].as_str().unwrap();
    let lines: Vec<&str> = csv_str.lines().collect();
    assert_eq!(lines.len(), 1, "expected only header row after clear");
    let cols: Vec<&str> = lines[0].split(',').collect();
    assert_eq!(cols.len(), 34, "CSV header must have exactly 34 columns");
    assert_eq!(cols[0], "measurement_uuid");
}

#[tokio::test]
async fn history_fifo_eviction() {
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

    // Take 1002 reads (cap is 1000).
    for _ in 0..1002 {
        module
            .handle_command("read", serde_json::json!({ "meter_id": meter_id }))
            .unwrap();
    }

    let json_export = module
        .handle_command("export_json", serde_json::json!({}))
        .unwrap();
    let json_str = json_export["json"].as_str().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
    assert_eq!(arr.len(), 1000, "history should cap at 1000");
}

#[tokio::test]
async fn export_json_validates_against_schema() {
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

    let json_export = module
        .handle_command("export_json", serde_json::json!({}))
        .unwrap();
    let json_str = json_export["json"].as_str().unwrap();

    module_meter::export::validate_export_json(json_str)
        .expect("exported JSON should validate against Phase 1 schema");
}

#[tokio::test]
async fn export_csv_round_trip() {
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

    let csv_export = module
        .handle_command("export_csv", serde_json::json!({}))
        .unwrap();
    let csv_str = csv_export["csv"].as_str().unwrap();

    let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());
    let headers: Vec<String> = rdr.headers().unwrap().iter().map(|s| s.to_string()).collect();
    assert!(headers.contains(&"measurement_uuid".to_string()));
    assert!(headers.contains(&"cct".to_string()));
    assert!(headers.contains(&"duv".to_string()));

    let records: Vec<csv::StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
    assert_eq!(records.len(), 1);

    // Verify cct and duv are non-empty (computed at construction time).
    let cct_idx = headers.iter().position(|h| h == "cct").unwrap();
    let duv_idx = headers.iter().position(|h| h == "duv").unwrap();
    assert!(!records[0][cct_idx].is_empty(), "cct should be populated");
    assert!(!records[0][duv_idx].is_empty(), "duv should be populated");
}

#[tokio::test]
async fn export_json_roundtrip_preserves_data() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let values: Vec<color_science::types::Xyz> = vec![
        color_science::types::Xyz { x: 10.0, y: 20.0, z: 30.0 },
        color_science::types::Xyz { x: 11.0, y: 21.0, z: 31.0 },
        color_science::types::Xyz { x: 12.0, y: 22.0, z: 32.0 },
        color_science::types::Xyz { x: 13.0, y: 23.0, z: 33.0 },
        color_science::types::Xyz { x: 14.0, y: 24.0, z: 34.0 },
    ];

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "Sequence": {
                        "values": [
                            { "Ok": values[0] },
                            { "Ok": values[1] },
                            { "Ok": values[2] },
                            { "Ok": values[3] },
                            { "Ok": values[4] }
                        ],
                        "loop_at_end": false
                    }
                }
            }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    for _ in 0..5 {
        module
            .handle_command("read", serde_json::json!({ "meter_id": meter_id }))
            .unwrap();
    }

    let json_export = module
        .handle_command("export_json", serde_json::json!({}))
        .unwrap();
    let json_str = json_export["json"].as_str().unwrap();

    let parsed: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
    assert_eq!(parsed.len(), 5);

    for (i, item) in parsed.iter().enumerate() {
        let uuid = item["measurementUuid"].as_str().unwrap();
        assert!(!uuid.is_empty(), "item {} should have a UUID", i);

        let x = item["xyz"]["x"].as_f64().unwrap();
        let y = item["xyz"]["y"].as_f64().unwrap();
        let z = item["xyz"]["z"].as_f64().unwrap();

        // JSON round-trip of f64 may introduce tiny epsilon; compare with tolerance.
        assert!(
            (x - values[i].x).abs() < 1e-9,
            "item {} x mismatch: expected {}, got {}",
            i,
            values[i].x,
            x
        );
        assert!(
            (y - values[i].y).abs() < 1e-9,
            "item {} y mismatch: expected {}, got {}",
            i,
            values[i].y,
            y
        );
        assert!(
            (z - values[i].z).abs() < 1e-9,
            "item {} z mismatch: expected {}, got {}",
            i,
            values[i].z,
            z
        );
    }
}

#[tokio::test]
async fn export_schema_patch_correlation_truth_table() {
    // Load the Phase 1 schema directly from disk (same file export.rs embeds).
    let schema_str =
        std::fs::read_to_string("../../docs/schemas/meter-export-phase1.json").unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

    // Helper: build a minimal conformant object with patch fields parameterized.
    let base = || serde_json::json!({
        "measurementUuid": "550e8400-e29b-41d4-a716-446655440000",
        "schemaVersion": "1.0",
        "softwareVersion": "2.0.0-phase1",
        "timestamp": "2026-04-30T12:00:00.123Z",
        "mode": "Emissive",
        "instrument": { "model": "FakeMeter", "id": "mock:1" },
        "xyz": { "x": 76.037, "y": 80.0, "z": 87.106 },
        "xyy": { "x": 0.3127, "y": 0.3290, "yLum": 80.0 },
        "lab": { "l": 83.138, "a": 0.0, "b": -1.803 },
        "lch": { "l": 83.138, "c": 1.803, "h": 270.0 },
        "uvPrime": { "u": 0.1978, "v": 0.4683 },
        "cct": 6504.0,
        "duv": 0.0,
        "deltaE2000": null,
        "target": null,
        "patchRgb": null,
        "patchBitDepth": null,
        "patchColorspace": "",
        "referenceWhite": "D65",
        "sessionId": null,
        "sequenceIndex": null,
        "label": ""
    });

    // ── Valid cases ─────────────────────────────────────────────

    // Case 1: real colorspace + object RGB + integer bit depth → valid
    let mut case1 = base();
    case1["patchColorspace"] = "BT.709".into();
    case1["patchRgb"] = serde_json::json!({ "r": 52428, "g": 52428, "b": 52428 });
    case1["patchBitDepth"] = serde_json::json!(16);
    jsonschema::validate(&schema, &case1).expect("case 1 (real colorspace + object RGB + integer) should be valid");

    // Case 2: empty colorspace + null RGB + null bit depth → valid
    let case2 = base();
    jsonschema::validate(&schema, &case2).expect("case 2 (empty colorspace + null patch) should be valid");

    // ── Rejected cases ──────────────────────────────────────────

    // Case 3: real colorspace + null RGB + null bit depth → rejected
    let mut case3 = base();
    case3["patchColorspace"] = "BT.709".into();
    assert!(
        jsonschema::validate(&schema, &case3).is_err(),
        "case 3 (real colorspace + null patch) should be rejected"
    );

    // Case 4: empty colorspace + object RGB + integer bit depth → rejected
    let mut case4 = base();
    case4["patchRgb"] = serde_json::json!({ "r": 52428, "g": 52428, "b": 52428 });
    case4["patchBitDepth"] = serde_json::json!(16);
    assert!(
        jsonschema::validate(&schema, &case4).is_err(),
        "case 4 (empty colorspace + object patch) should be rejected"
    );

    // Case 5: real colorspace + object RGB + null bit depth → rejected
    let mut case5 = base();
    case5["patchColorspace"] = "BT.709".into();
    case5["patchRgb"] = serde_json::json!({ "r": 52428, "g": 52428, "b": 52428 });
    assert!(
        jsonschema::validate(&schema, &case5).is_err(),
        "case 5 (real colorspace + object RGB + null bitDepth) should be rejected"
    );

    // Case 6: empty colorspace + null RGB + integer bit depth → rejected
    let mut case6 = base();
    case6["patchBitDepth"] = serde_json::json!(8);
    assert!(
        jsonschema::validate(&schema, &case6).is_err(),
        "case 6 (empty colorspace + null RGB + integer bitDepth) should be rejected"
    );
}

// ── list_active tests ──────────────────────────────────────────────

#[tokio::test]
async fn list_active_empty() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let result = module
        .handle_command("list_active", serde_json::json!({}))
        .unwrap();
    let meters = result["meters"].as_array().unwrap();
    assert!(meters.is_empty(), "expected no active meters");
}

#[tokio::test]
async fn list_active_one_connected() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let connect = module
        .handle_command("connect", serde_json::json!({ "instrument_id": "fake-meter-1" }))
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    let result = module
        .handle_command("list_active", serde_json::json!({}))
        .unwrap();
    let meters = result["meters"].as_array().unwrap();
    assert_eq!(meters.len(), 1);

    let entry = &meters[0];
    assert_eq!(entry["meter_id"].as_str().unwrap(), meter_id);
    assert_eq!(entry["instrument_model"].as_str().unwrap(), "FakeMeter");
    assert_eq!(entry["instrument_serial"].as_str().unwrap(), "fake-meter-1");
    assert!(
        entry["connected_at_utc"].as_str().unwrap().contains("T"),
        "connected_at_utc should be ISO 8601"
    );
    assert_eq!(entry["mode"].as_str().unwrap(), "Emissive");
    assert_eq!(entry["is_continuous_active"].as_bool().unwrap(), false);
}

#[tokio::test]
async fn list_active_continuous_flag() {
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

    let result = module
        .handle_command("list_active", serde_json::json!({}))
        .unwrap();
    let meters = result["meters"].as_array().unwrap();
    assert_eq!(meters[0]["is_continuous_active"].as_bool().unwrap(), true);

    module
        .handle_command("stop_continuous", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();

    let result2 = module
        .handle_command("list_active", serde_json::json!({}))
        .unwrap();
    let meters2 = result2["meters"].as_array().unwrap();
    assert_eq!(meters2[0]["is_continuous_active"].as_bool().unwrap(), false);
}

// ── probe tests ────────────────────────────────────────────────────

#[tokio::test]
async fn probe_result_shape() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let connect = module
        .handle_command("connect", serde_json::json!({ "instrument_id": "fake-meter-1" }))
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    let result = module
        .handle_command("probe", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();

    assert_eq!(result["responsive"].as_bool().unwrap(), true);
    assert_eq!(
        result["firmware_version"].as_str().unwrap(),
        "FakeMeter/1.0"
    );
    assert!(
        result["last_communication_utc"].as_str().unwrap().contains("T"),
        "last_communication_utc should be ISO 8601"
    );
}

#[tokio::test]
async fn probe_does_not_consume_sequence() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let xyz0 = color_science::types::Xyz { x: 1.0, y: 1.0, z: 1.0 };
    let xyz1 = color_science::types::Xyz { x: 2.0, y: 2.0, z: 2.0 };

    let connect = module
        .handle_command(
            "connect",
            serde_json::json!({
                "instrument_id": "fake-meter-1",
                "fake_meter_config": {
                    "Sequence": {
                        "values": [
                            { "Ok": xyz0 },
                            { "Ok": xyz1 }
                        ],
                        "loop_at_end": false
                    }
                }
            }),
        )
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    // Probe must NOT consume a sequence entry.
    module
        .handle_command("probe", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();

    // First read MUST return xyz0 (index 0), not xyz1.
    let read_result = module
        .handle_command("read", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();
    let measurement: color_science::measurement::MeasurementResult =
        serde_json::from_value(read_result).unwrap();

    assert!(
        (measurement.xyz.x - xyz0.x).abs() < 1e-9,
        "expected xyz0.x ({}), got {}",
        xyz0.x,
        measurement.xyz.x
    );
    assert!(
        (measurement.xyz.y - xyz0.y).abs() < 1e-9,
        "expected xyz0.y ({}), got {}",
        xyz0.y,
        measurement.xyz.y
    );
    assert!(
        (measurement.xyz.z - xyz0.z).abs() < 1e-9,
        "expected xyz0.z ({}), got {}",
        xyz0.z,
        measurement.xyz.z
    );
}

#[tokio::test]
async fn probe_allowed_during_continuous() {
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

    module
        .handle_command("probe", serde_json::json!({ "meter_id": meter_id }))
        .expect("probe should succeed during continuous read");

    module
        .handle_command("stop_continuous", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();
}

// ── get_config / set_config tests ──────────────────────────────────

#[tokio::test]
async fn get_set_config_roundtrip() {
    let event_bus = Arc::new(app_core::EventBus::new());
    let ctx = app_core::ModuleContext::new(event_bus.clone());
    let mut module = module_meter::MeterModule::new();
    module.initialize(&ctx).unwrap();

    let connect = module
        .handle_command("connect", serde_json::json!({ "instrument_id": "fake-meter-1" }))
        .unwrap();
    let meter_id = connect["meter_id"].as_str().unwrap();

    // Default config
    let default = module
        .handle_command("get_config", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();
    assert_eq!(default["mode"].as_str().unwrap(), "Emissive");
    assert_eq!(default["averaging_count"].as_u64().unwrap(), 1);
    assert!(default["integration_time_ms"].is_null());

    // Set new config
    let new_config = serde_json::json!({
        "meter_id": meter_id,
        "config": {
            "mode": "Ambient",
            "averaging_count": 3,
            "integration_time_ms": 250
        }
    });
    let set = module
        .handle_command("set_config", new_config)
        .unwrap();
    assert_eq!(set["mode"].as_str().unwrap(), "Ambient");
    assert_eq!(set["averaging_count"].as_u64().unwrap(), 3);
    assert_eq!(set["integration_time_ms"].as_u64().unwrap(), 250);

    // Verify persisted
    let retrieved = module
        .handle_command("get_config", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();
    assert_eq!(retrieved["mode"].as_str().unwrap(), "Ambient");
    assert_eq!(retrieved["averaging_count"].as_u64().unwrap(), 3);
    assert_eq!(retrieved["integration_time_ms"].as_u64().unwrap(), 250);
}

#[tokio::test]
async fn set_config_rejected_during_continuous() {
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
        .handle_command(
            "set_config",
            serde_json::json!({
                "meter_id": meter_id,
                "config": {
                    "mode": "Ambient",
                    "averaging_count": 1,
                    "integration_time_ms": null
                }
            }),
        )
        .expect_err("set_config should be rejected during continuous read");

    let msg = err.to_string();
    assert!(
        msg.contains("rejected") || msg.contains("continuous"),
        "expected rejection message, got: {}",
        msg
    );

    module
        .handle_command("stop_continuous", serde_json::json!({ "meter_id": meter_id }))
        .unwrap();
}

#[tokio::test]
async fn config_changed_event_emitted() {
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
        .handle_command(
            "set_config",
            serde_json::json!({
                "meter_id": meter_id,
                "config": {
                    "mode": "Ambient",
                    "averaging_count": 3,
                    "integration_time_ms": 250
                }
            }),
        )
        .unwrap();

    let event = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("should receive event")
        .expect("channel open");

    assert!(
        matches!(
            event,
            app_core::ModuleEvent::ConfigChanged {
                meter_id: ref mid,
                config,
            } if mid == meter_id
                && config.mode == hal::meter::MeasurementMode::Ambient
                && config.averaging_count == 3
                && config.integration_time_ms == Some(250)
        ),
        "expected ConfigChanged with correct meter_id and config, got {:?}",
        event
    );
}
