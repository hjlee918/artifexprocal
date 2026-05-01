//! Error types for the module system.

use std::fmt;

/// Error returned when a command handler fails.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    ModuleNotFound(String),
    UnknownCommand(String),
    InvalidPayload(String),
    ExecutionFailed(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::ModuleNotFound(id) => write!(f, "Module not found: {}", id),
            CommandError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            CommandError::InvalidPayload(msg) => write!(f, "Invalid payload: {}", msg),
            CommandError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// Error returned during module lifecycle operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleError {
    InitializationFailed(String),
    ActivationFailed(String),
    DeactivationFailed(String),
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            ModuleError::ActivationFailed(msg) => write!(f, "Activation failed: {}", msg),
            ModuleError::DeactivationFailed(msg) => write!(f, "Deactivation failed: {}", msg),
        }
    }
}

impl std::error::Error for ModuleError {}
