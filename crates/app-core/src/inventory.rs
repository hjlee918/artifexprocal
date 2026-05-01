//! Device inventory — list of known devices with connection history.
//!
//! Stub for Phase 1. Full SQLite-backed inventory in Phase 2.

use std::collections::HashMap;
use std::sync::RwLock;

/// A known device in the inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceRecord {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub connection_history: Vec<ConnectionStatus>,
}

/// Connection status for a device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

/// In-memory device inventory.
#[derive(Default)]
pub struct DeviceInventory {
    devices: RwLock<HashMap<String, DeviceRecord>>,
}

impl DeviceInventory {
    /// List all known devices.
    pub fn list(&self) -> Vec<DeviceRecord> {
        let devices = self.devices.read().unwrap();
        devices.values().cloned().collect()
    }

    /// Add a device to the inventory.
    pub fn add(&self, device: DeviceRecord) {
        let mut devices = self.devices.write().unwrap();
        devices.insert(device.id.clone(), device);
    }

    /// Remove a device by ID.
    pub fn remove(&self, id: &str) -> bool {
        let mut devices = self.devices.write().unwrap();
        devices.remove(id).is_some()
    }
}
