use calibration_core::state::{CalibrationTier, SessionConfig, TargetSpace, ToneCurve, WhitePoint};
use calibration_storage::query::{SessionFilter, SessionQuery};
use calibration_storage::schema::Storage;
use calibration_storage::session_store::SessionStore;
use calibration_storage::reading_store::ReadingStore;
use color_science::types::XYZ;
use rusqlite::params;

fn make_config(name: &str, target: TargetSpace) -> SessionConfig {
    SessionConfig {
        name: name.to_string(),
        target_space: target,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 5,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
            manual_patches: None,
    }
}

#[test]
fn test_empty_database_returns_empty_list() {
    let storage = Storage::new_in_memory().unwrap();
    let query = SessionQuery::new(&storage.conn);

    let (items, total) = query.list(&SessionFilter::default(), 0, 10).unwrap();
    assert!(items.is_empty());
    assert_eq!(total, 0);
}

#[test]
fn test_sessions_ordered_by_date_descending() {
    let storage = Storage::new_in_memory().unwrap();
    let store = SessionStore::new(&storage.conn);

    let id1 = store.create(&make_config("First", TargetSpace::Bt709)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let id2 = store.create(&make_config("Second", TargetSpace::Bt709)).unwrap();

    let query = SessionQuery::new(&storage.conn);
    let (items, _total) = query.list(&SessionFilter::default(), 0, 10).unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, id2); // Second created = first in DESC order
    assert_eq!(items[1].id, id1);
}

#[test]
fn test_filter_by_target_space() {
    let storage = Storage::new_in_memory().unwrap();
    let store = SessionStore::new(&storage.conn);

    let id1 = store.create(&make_config("BT709", TargetSpace::Bt709)).unwrap();
    let _id2 = store.create(&make_config("BT2020", TargetSpace::Bt2020)).unwrap();

    let query = SessionQuery::new(&storage.conn);
    let filter = SessionFilter {
        target_space: Some("BT.709".to_string()),
        ..SessionFilter::default()
    };
    let (items, total) = query.list(&filter, 0, 10).unwrap();

    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, id1);
}

#[test]
fn test_filter_by_state() {
    let storage = Storage::new_in_memory().unwrap();
    let store = SessionStore::new(&storage.conn);

    let id1 = store.create(&make_config("Running", TargetSpace::Bt709)).unwrap();
    let _id2 = store.create(&make_config("Idle", TargetSpace::Bt709)).unwrap();
    store.update_state(&id1, "finished").unwrap();

    let query = SessionQuery::new(&storage.conn);
    let filter = SessionFilter {
        state: Some("finished".to_string()),
        ..SessionFilter::default()
    };
    let (items, total) = query.list(&filter, 0, 10).unwrap();

    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, id1);
}

#[test]
fn test_pagination() {
    let storage = Storage::new_in_memory().unwrap();
    let store = SessionStore::new(&storage.conn);

    for i in 0..5 {
        store.create(&make_config(&format!("Session {}", i), TargetSpace::Bt709)).unwrap();
    }

    let query = SessionQuery::new(&storage.conn);
    let (page1, total) = query.list(&SessionFilter::default(), 0, 2).unwrap();
    let (page2, _total2) = query.list(&SessionFilter::default(), 1, 2).unwrap();

    assert_eq!(total, 5);
    assert_eq!(page1.len(), 2);
    assert_eq!(page2.len(), 2);
}

#[test]
fn test_get_detail_missing_returns_none() {
    let storage = Storage::new_in_memory().unwrap();
    let query = SessionQuery::new(&storage.conn);

    let result = query.get_detail("nonexistent-id").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_detail_returns_readings() {
    let storage = Storage::new_in_memory().unwrap();
    let store = SessionStore::new(&storage.conn);
    let reading_store = ReadingStore::new(&storage.conn);

    let config = SessionConfig {
        name: "Detail Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 2,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
            manual_patches: None,
    };

    let id = store.create(&config).unwrap();

    // Insert patches directly (needed for target_rgb enrichment)
    storage.conn.execute(
        "INSERT INTO patches (session_id, patch_index, patch_type, target_rgb) VALUES (?1, ?2, ?3, ?4)",
        params![&id, 0, "grayscale", "[0.0, 0.0, 0.0]"],
    ).unwrap();
    storage.conn.execute(
        "INSERT INTO patches (session_id, patch_index, patch_type, target_rgb) VALUES (?1, ?2, ?3, ?4)",
        params![&id, 1, "grayscale", "[1.0, 1.0, 1.0]"],
    ).unwrap();

    // Insert readings
    let xyz1 = XYZ { x: 0.5, y: 0.5, z: 0.5 };
    let xyz2 = XYZ { x: 95.0, y: 100.0, z: 108.0 };
    reading_store.save(&id, 0, 0, &xyz1, "cal").unwrap();
    reading_store.save(&id, 1, 0, &xyz2, "cal").unwrap();

    // Insert computed results
    storage.conn.execute(
        "INSERT INTO computed_results (session_id, gamma, max_de, avg_de, computed_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&id, 2.4, 1.23, 0.45, 1714320000000i64],
    ).unwrap();

    let query = SessionQuery::new(&storage.conn);
    let detail = query.get_detail(&id).unwrap().expect("detail should exist");

    assert_eq!(detail.summary.name, "Detail Test");
    assert_eq!(detail.readings.len(), 2);
    assert_eq!(detail.readings[0].patch_index, 0);
    assert_eq!(detail.readings[0].target_rgb, (0.0, 0.0, 0.0));
    assert_eq!(detail.readings[1].target_rgb, (1.0, 1.0, 1.0));
    assert!(detail.results.is_some());
    let results = detail.results.unwrap();
    assert_eq!(results.gamma, Some(2.4));
    assert_eq!(results.max_de, Some(1.23));
}
