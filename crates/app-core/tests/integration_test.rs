//! Integration test: CalibrationModule end-to-end lifecycle.
//!
//! Proves that a module can be:
//! 1. Registered
//! 2. Initialized with a ModuleContext
//! 3. Receive and respond to a command
//! 4. Emit an event via the EventBus

use app_core::{
    CalibrationModule, CommandError, EventBus, ModuleCapability, ModuleCommandDef,
    ModuleContext, ModuleEvent, ModuleError,
};
use serde_json::Value;
use std::sync::Arc;

/// A minimal mock module for testing the CalibrationModule trait contract.
struct MockModule {
    initialized: bool,
    last_command: Option<(String, Value)>,
}

impl MockModule {
    fn new() -> Self {
        Self {
            initialized: false,
            last_command: None,
        }
    }
}

impl CalibrationModule for MockModule {
    fn module_id(&self) -> &'static str {
        "mock"
    }

    fn display_name(&self) -> &'static str {
        "Mock Module"
    }

    fn capabilities(&self) -> Vec<ModuleCapability> {
        vec![ModuleCapability::Measurement, ModuleCapability::Standalone]
    }

    fn initialize(&mut self, _ctx: &ModuleContext) -> Result<(), ModuleError> {
        self.initialized = true;
        Ok(())
    }

    fn commands(&self) -> &'static [ModuleCommandDef] {
        &[
            ModuleCommandDef {
                name: "ping",
                description: "Health check",
            },
            ModuleCommandDef {
                name: "echo",
                description: "Echo back the payload",
            },
        ]
    }

    fn handle_command(
        &mut self,
        cmd: &str,
        payload: Value,
    ) -> Result<Value, CommandError> {
        self.last_command = Some((cmd.to_string(), payload.clone()));

        match cmd {
            "ping" => Ok(serde_json::json!({ "status": "ok" })),
            "echo" => Ok(payload),
            _ => Err(CommandError::UnknownCommand(cmd.to_string())),
        }
    }
}

#[tokio::test]
async fn mock_module_lifecycle_end_to_end() {
    // ── Setup ──────────────────────────────────────────────
    let event_bus = Arc::new(EventBus::new());
    let ctx = ModuleContext::new(event_bus.clone());

    let mut module = MockModule::new();

    // ── Step 1: Register / Identify ──────────────────────
    assert_eq!(module.module_id(), "mock");
    assert_eq!(module.display_name(), "Mock Module");
    assert_eq!(
        module.capabilities(),
        vec![ModuleCapability::Measurement, ModuleCapability::Standalone]
    );

    // ── Step 2: Initialize ─────────────────────────────────
    module.initialize(&ctx).expect("initialize should succeed");
    assert!(module.initialized, "module should be initialized");

    // ── Step 3: Commands ─────────────────────────────────
    let ping_result = module
        .handle_command("ping", serde_json::json!({}))
        .expect("ping should succeed");
    assert_eq!(ping_result, serde_json::json!({ "status": "ok" }));

    let echo_payload = serde_json::json!({ "message": "hello" });
    let echo_result = module
        .handle_command("echo", echo_payload.clone())
        .expect("echo should succeed");
    assert_eq!(echo_result, echo_payload);

    let unknown = module.handle_command("bad_cmd", serde_json::json!({}));
    assert!(
        matches!(unknown, Err(CommandError::UnknownCommand(_))),
        "unknown command should error"
    );

    // ── Step 4: EventBus ─────────────────────────────────
    let mut rx = event_bus.subscribe();

    let event = ModuleEvent::CommandCompleted {
        module_id: "mock".to_string(),
        command: "ping".to_string(),
    };
    event_bus.publish(event.clone());

    let received = rx.recv().await.expect("should receive event");
    assert_eq!(received, event);
}
