//! app-core — Core types, CalibrationModule trait, settings, event bus.

pub mod errors;
pub mod event_bus;
pub mod inventory;
pub mod module;
pub mod registers;
pub mod settings;

pub use errors::{CommandError, ModuleError};
pub use event_bus::{ContinuousReadStopReason, EventBus, ModuleEvent};
pub use inventory::DeviceInventory;
pub use module::{CalibrationModule, ModuleCapability, ModuleCommandDef, ModuleContext};
pub use registers::RegisterSlot;
pub use settings::SettingsStore;
