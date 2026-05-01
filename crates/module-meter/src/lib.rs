//! module-meter — MeterModule CalibrationModule implementation.

use app_core::{
    CalibrationModule, CommandError, ModuleCapability, ModuleCommandDef, ModuleContext,
    ModuleError,
};
use color_science::measurement::MeasurementResult;
use hal::meter::Meter;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ConnectRequest {
    instrument_id: String,
    fake_meter_config: Option<hal_meters::FakeMeterConfig>,
}

/// Runtime state for an actively connected meter.
struct ActiveMeter {
    #[allow(dead_code)]
    instrument_id: String,
    instrument_model: String,
    driver: Box<dyn Meter>,
}

/// MeterModule — implements CalibrationModule for instrument discovery,
/// connection, and measurement.
pub struct MeterModule {
    ctx: Option<ModuleContext>,
    active_meters: HashMap<String, ActiveMeter>,
}

impl MeterModule {
    pub fn new() -> Self {
        Self {
            ctx: None,
            active_meters: HashMap::new(),
        }
    }

    fn cmd_detect(&self) -> Result<Value, CommandError> {
        let instruments = vec![serde_json::json!({
            "id": "fake-meter-1",
            "model": "FakeMeter",
            "manufacturer": "ArtifexProCal",
            "instrument_type": "Colorimeter",
            "connection_method": "Mock",
            "native_driver_available": true,
        })];
        Ok(serde_json::to_value(instruments).unwrap())
    }

    fn cmd_connect(&mut self, payload: Value) -> Result<Value, CommandError> {
        let req: ConnectRequest = serde_json::from_value(payload)
            .map_err(|e| CommandError::InvalidPayload(e.to_string()))?;

        if req.instrument_id != "fake-meter-1" {
            return Err(CommandError::ExecutionFailed(format!(
                "instrument {} not found",
                req.instrument_id
            )));
        }

        let meter_id = uuid::Uuid::new_v4().to_string();
        let driver: Box<dyn Meter> = match req.fake_meter_config {
            Some(config) => Box::new(
                hal_meters::FakeMeter::with_config(config)
                    .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?,
            ),
            None => Box::new(hal_meters::FakeMeter::new()),
        };

        self.active_meters.insert(
            meter_id.clone(),
            ActiveMeter {
                instrument_id: req.instrument_id,
                instrument_model: "FakeMeter".to_string(),
                driver,
            },
        );

        Ok(serde_json::json!({ "meter_id": meter_id }))
    }

    fn cmd_disconnect(&mut self, payload: Value) -> Result<Value, CommandError> {
        let meter_id = payload
            .get("meter_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidPayload("missing meter_id".to_string()))?;

        let mut active = self
            .active_meters
            .remove(meter_id)
            .ok_or_else(|| CommandError::ExecutionFailed("meter not found".to_string()))?;

        active
            .driver
            .disconnect()
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        Ok(Value::Null)
    }

    fn cmd_read(&mut self, payload: Value) -> Result<Value, CommandError> {
        let meter_id = payload
            .get("meter_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidPayload("missing meter_id".to_string()))?;

        let active = self
            .active_meters
            .get_mut(meter_id)
            .ok_or_else(|| CommandError::ExecutionFailed("meter not found".to_string()))?;

        let xyz = active
            .driver
            .read_xyz()
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        let result = MeasurementResult::from_xyz(
            xyz,
            meter_id,
            active.instrument_id.clone(),
            active.instrument_model.clone(),
        );

        Ok(serde_json::to_value(result).unwrap())
    }
}

impl Default for MeterModule {
    fn default() -> Self {
        Self::new()
    }
}

impl CalibrationModule for MeterModule {
    fn module_id(&self) -> &'static str {
        "meter"
    }

    fn display_name(&self) -> &'static str {
        "Colorimeter / Spectrophotometer"
    }

    fn capabilities(&self) -> Vec<ModuleCapability> {
        vec![
            ModuleCapability::Measurement,
            ModuleCapability::Standalone,
            ModuleCapability::HardwareProbe,
        ]
    }

    fn initialize(&mut self, ctx: &ModuleContext) -> Result<(), ModuleError> {
        self.ctx = Some(ctx.clone());
        Ok(())
    }

    fn commands(&self) -> &'static [ModuleCommandDef] {
        &[
            ModuleCommandDef {
                name: "detect",
                description: "Enumerate connected instruments",
            },
            ModuleCommandDef {
                name: "connect",
                description: "Connect to an instrument by ID",
            },
            ModuleCommandDef {
                name: "disconnect",
                description: "Disconnect an instrument",
            },
            ModuleCommandDef {
                name: "read",
                description: "Take a single measurement",
            },
        ]
    }

    fn handle_command(&mut self, cmd: &str, payload: Value) -> Result<Value, CommandError> {
        match cmd {
            "detect" => self.cmd_detect(),
            "connect" => self.cmd_connect(payload),
            "disconnect" => self.cmd_disconnect(payload),
            "read" => self.cmd_read(payload),
            _ => Err(CommandError::UnknownCommand(cmd.to_string())),
        }
    }
}
