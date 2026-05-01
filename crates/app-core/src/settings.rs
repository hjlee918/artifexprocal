//! Settings store — SQLite-backed key-value store with typed getters/setters.
//!
//! This is a minimal stub for Phase 1. Full SQLite persistence will be
//! added when calibration-storage is wired up.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory settings store. Will be backed by SQLite in Phase 2.
#[derive(Default)]
pub struct SettingsStore {
    data: RwLock<HashMap<String, Value>>,
}

impl SettingsStore {
    /// Get a JSON value by key.
    pub fn get(&self, key: &str) -> Option<Value> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    /// Get and deserialize a JSON value.
    pub fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Option<T> {
        let data = self.data.read().unwrap();
        data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a JSON value by key.
    pub fn set(&self, key: &str, value: Value) {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), value);
    }

    /// Set a serializable value.
    pub fn set_json<T: Serialize>(&self, key: &str, value: &T) {
        let json = serde_json::to_value(value).unwrap();
        self.set(key, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip() {
        let store = SettingsStore::default();
        store.set("test.key", serde_json::json!(42));
        assert_eq!(store.get("test.key"), Some(serde_json::json!(42)));
    }

    #[test]
    fn settings_json_roundtrip() {
        let store = SettingsStore::default();
        store.set_json("test.vec", &vec![1, 2, 3]);
        let got: Vec<i32> = store.get_json("test.vec").unwrap();
        assert_eq!(got, vec![1, 2, 3]);
    }
}
