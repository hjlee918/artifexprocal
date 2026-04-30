use hal_meters::init_tracker::InitTracker;
use calibration_storage::schema::Storage;

#[test]
fn test_init_tracker_record_and_query() {
    let storage = Storage::new_in_memory().unwrap();
    InitTracker::record_init(&storage.conn, "SN12345", "i1 Pro 2").unwrap();

    let duration = InitTracker::time_until_next_init(&storage.conn, "SN12345").unwrap();
    assert!(duration.as_secs() > 10700); // > ~3h - 10s

    assert!(!InitTracker::is_init_expired(&storage.conn, "SN12345"));
}

#[test]
fn test_init_tracker_expired() {
    let storage = Storage::new_in_memory().unwrap();
    // No record exists — treat as expired
    assert!(InitTracker::is_init_expired(&storage.conn, "UNKNOWN"));
    assert_eq!(
        InitTracker::time_until_next_init(&storage.conn, "UNKNOWN"),
        None
    );
}
