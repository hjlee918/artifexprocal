//! CalibrationModule trait and supporting types.

use crate::errors::{CommandError, ModuleError};
use crate::event_bus::EventBus;
use crate::inventory::DeviceInventory;
use crate::settings::SettingsStore;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Capabilities a module can advertise to the workflow engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleCapability {
    Measurement,
    Standalone,
    HardwareProbe,
    DisplayControl,
    PatternGeneration,
    Profiling,
    Reporting,
}

/// Definition of a command exposed by a module.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCommandDef {
    pub name: &'static str,
    pub description: &'static str,
}

/// Context provided to every module at initialization.
///
/// No module holds a concrete reference to another module.
/// All inter-module communication goes through the event bus.
#[derive(Clone)]
pub struct ModuleContext {
    pub settings: Arc<SettingsStore>,
    pub inventory: Arc<DeviceInventory>,
    pub event_bus: Arc<EventBus>,
}

impl ModuleContext {
    /// Create a minimal context for testing.
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            settings: Arc::new(SettingsStore::default()),
            inventory: Arc::new(DeviceInventory::default()),
            event_bus,
        }
    }
}

/// Every calibration module implements this trait.
///
/// The trait is object-safe: all methods take &mut self and use
/// erased types for the return values.
pub trait CalibrationModule: Send {
    /// Unique module identifier (e.g., "meter", "display").
    fn module_id(&self) -> &'static str;

    /// Human-readable display name.
    fn display_name(&self) -> &'static str {
        self.module_id()
    }

    /// Capabilities advertised to the workflow engine.
    fn capabilities(&self) -> Vec<ModuleCapability> {
        Vec::new()
    }

    /// Lifecycle: called once when the module is registered at app startup.
    fn initialize(&mut self, _ctx: &ModuleContext) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Lifecycle: called when the module participates in an active workflow.
    fn activate(&mut self, _workflow_id: String) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Lifecycle: called when the workflow ends or the module is swapped out.
    fn deactivate(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Return the set of IPC commands this module exposes.
    fn commands(&self) -> &'static [ModuleCommandDef] {
        &[]
    }

    /// Handle a command invocation from the frontend.
    fn handle_command(
        &mut self,
        _cmd: &str,
        _payload: Value,
    ) -> Result<Value, CommandError> {
        Err(CommandError::UnknownCommand("no commands defined".to_string()))
    }

    /// Return an event-stream receiver, if the module emits events.
    fn event_stream(&self) -> Option<broadcast::Receiver<crate::event_bus::ModuleEvent>> {
        None
    }
}
