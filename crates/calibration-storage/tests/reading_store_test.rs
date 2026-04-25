use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint};
use calibration_storage::schema::Storage;
use calibration_storage::session_store::SessionStore;
use calibration_storage::reading_store::ReadingStore;
use color_science::types::XYZ;

#[test]
fn test_save_and_load_readings() {
    let storage = Storage::new_in_memory().unwrap();
    let session_store = SessionStore::new(&storage.conn);
    let reading_store = ReadingStore::new(&storage.conn);

    let config = SessionConfig {
        name: "Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 500,
        stability_threshold: None,
    };

    let session_id = session_store.create(&config).unwrap();

    let xyz = XYZ { x: 10.0, y: 20.0, z: 30.0 };
    reading_store.save(&session_id, 0, 0, &xyz, "cal").unwrap();
    reading_store.save(&session_id, 0, 1, &xyz, "cal").unwrap();

    let readings = reading_store.load_for_patch(&session_id, 0, "cal").unwrap();
    assert_eq!(readings.len(), 2);
    assert_eq!(readings[0].x, 10.0);
}
