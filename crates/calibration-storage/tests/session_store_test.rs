use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint, CalibrationTier};
use calibration_storage::schema::Storage;
use calibration_storage::session_store::SessionStore;

#[test]
fn test_create_and_get_session() {
    let storage = Storage::new_in_memory().unwrap();
    let store = SessionStore::new(&storage.conn);

    let config = SessionConfig {
        name: "Test Session".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 500,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
    };

    let id = store.create(&config).unwrap();
    let session = store.get(&id).unwrap();
    assert_eq!(session.name, "Test Session");
    assert_eq!(session.target_space, "BT.709");
}

#[test]
fn test_update_session_state() {
    let storage = Storage::new_in_memory().unwrap();
    let store = SessionStore::new(&storage.conn);

    let config = SessionConfig {
        name: "Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 500,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
    };

    let id = store.create(&config).unwrap();
    store.update_state(&id, "measuring").unwrap();

    let session = store.get(&id).unwrap();
    assert_eq!(session.state, "measuring");
}
