use calibration_storage::schema::Storage;
use std::path::PathBuf;

#[test]
fn test_storage_in_memory() {
    let _storage = Storage::new_in_memory().unwrap();
}

#[test]
fn test_storage_file_based() {
    let temp_path = PathBuf::from("/tmp/test_cal_storage.db");
    let _ = std::fs::remove_file(&temp_path);
    let _storage = Storage::new(&temp_path).unwrap();
    assert!(temp_path.exists());
    let _ = std::fs::remove_file(&temp_path);
}
