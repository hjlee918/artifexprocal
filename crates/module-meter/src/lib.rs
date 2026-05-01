//! module-meter — MeterModule CalibrationModule implementation.

use app_core::{
    CalibrationModule, CommandError, ContinuousReadStopReason, EventBus, ModuleCapability,
    ModuleCommandDef, ModuleContext, ModuleError, ModuleEvent, RegisterSlot,
};
use color_science::measurement::MeasurementResult;
use hal::meter::Meter;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

pub mod export;

#[derive(Debug, Deserialize)]
struct ConnectRequest {
    instrument_id: String,
    fake_meter_config: Option<hal_meters::FakeMeterConfig>,
}

#[derive(Debug, Deserialize)]
struct ReadContinuousRequest {
    meter_id: String,
    interval_ms: u32,
}

#[derive(Debug, Deserialize)]
struct StopContinuousRequest {
    meter_id: String,
}

#[derive(Debug, Deserialize)]
struct SetRegisterRequest {
    slot: RegisterSlot,
    measurement: MeasurementResult,
}

#[derive(Debug, Deserialize)]
struct ClearRegisterRequest {
    slot: RegisterSlot,
}

/// Runtime state for an actively connected meter.
struct ActiveMeter {
    instrument_id: String,
    instrument_model: String,
    driver: Arc<Mutex<Box<dyn Meter>>>,
}

/// State for a running continuous read loop.
struct ContinuousReadState {
    cancel_token: CancellationToken,
    #[allow(dead_code)]
    handle: tokio::task::JoinHandle<()>,
}

/// MeterModule — implements CalibrationModule for instrument discovery,
/// connection, and measurement.
pub struct MeterModule {
    ctx: Option<ModuleContext>,
    active_meters: HashMap<String, ActiveMeter>,
    continuous_reads: HashMap<String, ContinuousReadState>,
    registers: Arc<Mutex<HashMap<RegisterSlot, MeasurementResult>>>,
    history: Arc<Mutex<VecDeque<MeasurementResult>>>,
}

impl MeterModule {
    pub fn new() -> Self {
        Self {
            ctx: None,
            active_meters: HashMap::new(),
            continuous_reads: HashMap::new(),
            registers: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn event_bus(&self) -> Arc<EventBus> {
        self.ctx
            .as_ref()
            .expect("module not initialized")
            .event_bus
            .clone()
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
                driver: Arc::new(Mutex::new(driver)),
            },
        );

        Ok(serde_json::json!({ "meter_id": meter_id }))
    }

    fn cmd_disconnect(&mut self, payload: Value) -> Result<Value, CommandError> {
        let meter_id = payload
            .get("meter_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidPayload("missing meter_id".to_string()))?;

        // Cancel any active continuous read (fire-and-forget).
        if let Some(state) = self.continuous_reads.remove(meter_id) {
            state.cancel_token.cancel();
        }

        let active = self
            .active_meters
            .remove(meter_id)
            .ok_or_else(|| CommandError::ExecutionFailed("meter not found".to_string()))?;

        active
            .driver
            .lock()
            .unwrap()
            .disconnect()
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        Ok(Value::Null)
    }

    fn cmd_read(&mut self, payload: Value) -> Result<Value, CommandError> {
        let meter_id = payload
            .get("meter_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::InvalidPayload("missing meter_id".to_string()))?;

        if self.continuous_reads.contains_key(meter_id) {
            return Err(CommandError::ExecutionFailed(
                "single read rejected while continuous read is active".to_string(),
            ));
        }

        let active = self
            .active_meters
            .get(meter_id)
            .ok_or_else(|| CommandError::ExecutionFailed("meter not found".to_string()))?;

        let xyz = active
            .driver
            .lock()
            .unwrap()
            .read_xyz()
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        let result = MeasurementResult::from_xyz(
            xyz,
            meter_id,
            active.instrument_id.clone(),
            active.instrument_model.clone(),
        );

        // Auto-populate Current register silently (no event).
        self.registers
            .lock()
            .unwrap()
            .insert(RegisterSlot::Current, result.clone());

        export::push_history(&mut self.history.lock().unwrap(), result.clone());

        self.event_bus().publish(ModuleEvent::MeasurementReceived {
            meter_id: meter_id.to_string(),
            measurement: result.clone(),
        });

        Ok(serde_json::to_value(result).unwrap())
    }

    fn cmd_read_continuous(&mut self, payload: Value) -> Result<Value, CommandError> {
        let req: ReadContinuousRequest = serde_json::from_value(payload)
            .map_err(|e| CommandError::InvalidPayload(e.to_string()))?;

        let active = self
            .active_meters
            .get(&req.meter_id)
            .ok_or_else(|| CommandError::ExecutionFailed("meter not found".to_string()))?;

        if self.continuous_reads.contains_key(&req.meter_id) {
            return Err(CommandError::ExecutionFailed(
                "continuous read already active".to_string(),
            ));
        }

        let cancel_token = CancellationToken::new();
        let child_token = cancel_token.child_token();
        let meter_id = req.meter_id.clone();
        let event_bus = self.event_bus();
        let registers = Arc::clone(&self.registers);
        let history = Arc::clone(&self.history);
        let interval_ms = req.interval_ms;
        let active = active.clone();

        let handle = tokio::spawn(async move {
            continuous_read_loop(
                meter_id,
                active.instrument_id,
                active.instrument_model,
                active.driver,
                event_bus,
                registers,
                history,
                interval_ms,
                child_token,
            )
            .await;
        });

        self.continuous_reads.insert(
            req.meter_id,
            ContinuousReadState {
                cancel_token,
                handle,
            },
        );

        Ok(serde_json::json!({ "status": "started" }))
    }

    fn cmd_stop_continuous(&mut self, payload: Value) -> Result<Value, CommandError> {
        let req: StopContinuousRequest = serde_json::from_value(payload)
            .map_err(|e| CommandError::InvalidPayload(e.to_string()))?;

        let state = self
            .continuous_reads
            .remove(&req.meter_id)
            .ok_or_else(|| CommandError::ExecutionFailed(
                "no continuous read active for meter".to_string(),
            ))?;

        state.cancel_token.cancel();

        Ok(serde_json::json!({ "status": "stopped" }))
    }

    fn cmd_set_register(&mut self, payload: Value) -> Result<Value, CommandError> {
        let req: SetRegisterRequest = serde_json::from_value(payload)
            .map_err(|e| CommandError::InvalidPayload(e.to_string()))?;

        self.registers
            .lock()
            .unwrap()
            .insert(req.slot, req.measurement.clone());

        self.event_bus().publish(ModuleEvent::RegisterChanged {
            slot: req.slot,
            measurement: Some(req.measurement),
        });

        Ok(Value::Null)
    }

    fn cmd_clear_register(&mut self, payload: Value) -> Result<Value, CommandError> {
        let req: ClearRegisterRequest = serde_json::from_value(payload)
            .map_err(|e| CommandError::InvalidPayload(e.to_string()))?;

        self.registers.lock().unwrap().remove(&req.slot);

        self.event_bus().publish(ModuleEvent::RegisterChanged {
            slot: req.slot,
            measurement: None,
        });

        Ok(Value::Null)
    }

    fn cmd_get_all_registers(&self) -> Result<Value, CommandError> {
        let map = self.registers.lock().unwrap().clone();
        Ok(serde_json::to_value(map).unwrap())
    }

    fn cmd_export_json(&self) -> Result<Value, CommandError> {
        let mut history = self.history.lock().unwrap();
        let json = export::export_json(history.make_contiguous())
            .map_err(|e| CommandError::ExecutionFailed(format!("export json failed: {}", e)))?;
        Ok(serde_json::json!({ "json": json }))
    }

    fn cmd_export_csv(&self) -> Result<Value, CommandError> {
        let mut history = self.history.lock().unwrap();
        let csv = export::export_csv(history.make_contiguous())
            .map_err(|e| CommandError::ExecutionFailed(format!("export csv failed: {}", e)))?;
        Ok(serde_json::json!({ "csv": csv }))
    }

    fn cmd_clear_history(&mut self) -> Result<Value, CommandError> {
        self.history.lock().unwrap().clear();
        Ok(Value::Null)
    }
}

impl Clone for ActiveMeter {
    fn clone(&self) -> Self {
        Self {
            instrument_id: self.instrument_id.clone(),
            instrument_model: self.instrument_model.clone(),
            driver: Arc::clone(&self.driver),
        }
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
            ModuleCommandDef {
                name: "read_continuous",
                description: "Start continuous measurement",
            },
            ModuleCommandDef {
                name: "stop_continuous",
                description: "Stop continuous measurement",
            },
            ModuleCommandDef {
                name: "set_register",
                description: "Store a measurement into a register slot",
            },
            ModuleCommandDef {
                name: "clear_register",
                description: "Clear a register slot",
            },
            ModuleCommandDef {
                name: "get_all_registers",
                description: "Return all populated registers",
            },
            ModuleCommandDef {
                name: "export_json",
                description: "Export measurement history as JSON",
            },
            ModuleCommandDef {
                name: "export_csv",
                description: "Export measurement history as CSV",
            },
            ModuleCommandDef {
                name: "clear_history",
                description: "Clear measurement history",
            },
        ]
    }

    fn handle_command(
        &mut self,
        cmd: &str,
        payload: Value,
    ) -> Result<Value, CommandError> {
        match cmd {
            "detect" => self.cmd_detect(),
            "connect" => self.cmd_connect(payload),
            "disconnect" => self.cmd_disconnect(payload),
            "read" => self.cmd_read(payload),
            "read_continuous" => self.cmd_read_continuous(payload),
            "stop_continuous" => self.cmd_stop_continuous(payload),
            "set_register" => self.cmd_set_register(payload),
            "clear_register" => self.cmd_clear_register(payload),
            "get_all_registers" => self.cmd_get_all_registers(),
            "export_json" => self.cmd_export_json(),
            "export_csv" => self.cmd_export_csv(),
            "clear_history" => self.cmd_clear_history(),
            _ => Err(CommandError::UnknownCommand(cmd.to_string())),
        }
    }
}

async fn continuous_read_loop(
    meter_id: String,
    instrument_id: String,
    instrument_model: String,
    driver: Arc<Mutex<Box<dyn Meter>>>,
    event_bus: Arc<EventBus>,
    registers: Arc<Mutex<HashMap<RegisterSlot, MeasurementResult>>>,
    history: Arc<Mutex<VecDeque<MeasurementResult>>>,
    interval_ms: u32,
    cancel_token: CancellationToken,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval_ms as u64));
    let mut consecutive_errors: u32 = 0;

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                event_bus.publish(ModuleEvent::ContinuousReadStopped {
                    meter_id: meter_id.clone(),
                    reason: ContinuousReadStopReason::Cancelled,
                });
                break;
            }
            _ = interval.tick() => {
                let result = {
                    let mut guard = driver.lock().unwrap();
                    guard.read_xyz()
                };

                match result {
                    Ok(xyz) => {
                        consecutive_errors = 0;
                        let measurement = MeasurementResult::from_xyz(
                            xyz,
                            &meter_id,
                            &instrument_id,
                            &instrument_model,
                        );
                        // Auto-populate Current register silently (no event).
                        registers.lock().unwrap().insert(RegisterSlot::Current, measurement.clone());
                        export::push_history(&mut history.lock().unwrap(), measurement.clone());
                        event_bus.publish(ModuleEvent::MeasurementReceived {
                            meter_id: meter_id.clone(),
                            measurement,
                        });
                    }
                    Err(e) => {
                        consecutive_errors += 1;

                        if !e.is_transient() {
                            let reason = if matches!(e, hal::meter::MeterError::SequenceExhausted) {
                                ContinuousReadStopReason::SequenceExhausted
                            } else {
                                ContinuousReadStopReason::FatalError(e.to_string())
                            };
                            event_bus.publish(ModuleEvent::ContinuousReadStopped {
                                meter_id: meter_id.clone(),
                                reason,
                            });
                            break;
                        }

                        if consecutive_errors > 3 {
                            event_bus.publish(ModuleEvent::ContinuousReadStopped {
                                meter_id: meter_id.clone(),
                                reason: ContinuousReadStopReason::ErrorToleranceExceeded,
                            });
                            break;
                        }

                        event_bus.publish(ModuleEvent::ContinuousReadError {
                            meter_id: meter_id.clone(),
                            error: e.to_string(),
                            consecutive_count: consecutive_errors,
                        });
                    }
                }
            }
        }
    }
}
