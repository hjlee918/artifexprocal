# MeterModule Design Document

**Date:** 2026-04-30
**Status:** Draft — pending review
**Scope:** Complete design for the MeterModule — the first `CalibrationModule` implementation in the v2 architecture. Covers Rust backend, ArgyllCMS integration, instrument detection, `MeasurementResult` production, IPC surface, frontend UI, and adaptation from v1 `hal_meters`.

---

## 1. Module Overview

The MeterModule is responsible for:
1. **Discovery** — enumerate connected colorimeters and spectrophotometers via USB
2. **Connection** — open a communication channel (native HID/USB or ArgyllCMS subprocess)
3. **Configuration** — set measurement mode (emissive, ambient, etc.), integration time, and correction matrix
4. **Measurement** — trigger readings and produce `MeasurementResult`
5. **Monitoring** — emit real-time measurement events and instrument health status
6. **Standalone mode** — allow spot reads without an active workflow (e.g., for quick verification)

### 1.1 Supported Instruments

| Instrument | Type | Platform | Driver Path |
|-----------|------|----------|-------------|
| X-Rite i1 Display Pro Rev.B | Colorimeter | macOS | ArgyllCMS `spotread -c 1` via PTY |
| X-Rite i1 Display Pro Rev.B | Colorimeter | Linux/Windows | Native HID via `hidapi` + challenge-response unlock |
| X-Rite i1 Pro 2 | Spectrophotometer | macOS | ArgyllCMS `spotread -c 2` via PTY |
| X-Rite i1 Pro 2 | Spectrophotometer | Linux/Windows | Native USB bulk via `rusb` |
| X-Rite i1Display Studio | Colorimeter | All | ArgyllCMS (OEM variant of i1 Display Pro) |
| X-Rite ColorMunki Display | Colorimeter | All | ArgyllCMS |
| Datacolor SpyderX | Colorimeter | All | ArgyllCMS |
| Klein K-10A | Colorimeter | All | ArgyllCMS (if supported) |
| Jeti spectravo | Spectrophotometer | All | ArgyllCMS |

**Design principle:** ArgyllCMS is the universal fallback. Native drivers are implemented only when they provide clear advantages (speed, unlock, no AGPL dependency on macOS is not possible — ArgyllCMS is required there).

### 1.2 Platform Routing

| Platform | i1 Display Pro | i1 Pro 2 | Other |
|----------|---------------|----------|-------|
| macOS | ArgyllCMS `spotread` | ArgyllCMS `spotread` | ArgyllCMS `spotread` |
| Linux | Native HID | Native USB | ArgyllCMS |
| Windows | Native HID | Native USB | ArgyllCMS |

---

## 2. Rust Backend Design

### 2.1 V1 Crate Adaptation Strategy

The v1 `hal_meters` crate contains working, tested code:
- `I1DisplayPro` — native HID driver with challenge-response unlock
- `I1Pro2` — native USB bulk driver
- `ArgyllMeter` — PTY subprocess wrapper for `spotread`
- `i1d3_unlock` — 11 known OEM keys + `I1D3_ESCAPE` fallback

**The v2 `module-meter` crate does NOT rewrite these drivers.** It imports `hal-meters` as a dependency and wraps them.

```toml
# crates/module-meter/Cargo.toml
[dependencies]
hal-meters = { path = "../hal-meters" }
color-science = { path = "../color-science" }
app-core = { path = "../app-core" }
tokio = { version = "1", features = ["rt-multi-thread", "process", "time", "sync"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 2.2 Module Implementation

```rust
// crates/module-meter/src/meter_module.rs

use app_core::{CalibrationModule, ModuleContext, ModuleCapability, ModuleCommandDef, ModuleEvent};
use hal::meter::Meter;
use calibration_engine::EventBus;
use std::sync::{Arc, Mutex};

pub struct MeterModule {
    ctx: Option<ModuleContext>,
    active_meters: Vec<ActiveMeter>,
    detection_cache: Vec<DetectedInstrument>,
    config: MeterModuleConfig,
}

struct ActiveMeter {
    id: String,
    instrument: Box<dyn Meter>,
    config: MeterConfig,
    health: MeterHealth,
}

#[derive(Debug, Clone)]
pub struct MeterModuleConfig {
    pub default_integration_time_ms: u32,
    pub stabilization_delay_ms: u32,
    pub auto_dark_current: bool,
    pub preferred_driver: DriverPreference, // Native, ArgyllCMS, Auto
}

#[derive(Debug, Clone)]
pub struct MeterConfig {
    pub measurement_mode: MeasurementMode,
    pub integration_time_ms: Option<u32>,  // None = auto
    pub correction_matrix_id: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum MeasurementMode {
    Emissive,   // Display direct measurement (default)
    Ambient,    // Ambient light measurement
    Flash,      // Flash / projector measurement
    Telephoto,  // Measurement through lens
}

#[derive(Debug, Clone)]
pub struct MeterHealth {
    pub connected_at: DateTime<Utc>,
    pub last_reading_at: Option<DateTime<Utc>>,
    pub read_count: u64,
    pub error_count: u64,
    pub temperature_celsius: Option<f32>,
}
```

### 2.3 CalibrationModule Trait Implementation

```rust
impl CalibrationModule for MeterModule {
    fn module_id(&self) -> &'static str { "meter" }
    fn display_name(&self) -> &'static str { "Colorimeter / Spectrophotometer" }

    fn capabilities(&self) -> Vec<ModuleCapability> {
        vec![
            ModuleCapability::Measurement,
            ModuleCapability::Standalone,
            ModuleCapability::HardwareProbe,
        ]
    }

    fn initialize(&mut self, ctx: &ModuleContext) -> Result<(), ModuleError> {
        self.ctx = Some(ctx.clone());
        // Load saved meter config from settings
        let config: MeterModuleConfig = ctx.settings
            .get_json("meter.module_config")
            .unwrap_or_default();
        self.config = config;
        Ok(())
    }

    fn commands(&self) -> &'static [ModuleCommandDef] {
        &[
            ModuleCommandDef { name: "detect", description: "Enumerate connected instruments" },
            ModuleCommandDef { name: "connect", description: "Connect to an instrument by ID" },
            ModuleCommandDef { name: "disconnect", description: "Disconnect an instrument" },
            ModuleCommandDef { name: "read", description: "Take a single measurement" },
            ModuleCommandDef { name: "read_continuous", description: "Start continuous measurement" },
            ModuleCommandDef { name: "stop_continuous", description: "Stop continuous measurement" },
            ModuleCommandDef { name: "set_config", description: "Set meter configuration" },
            ModuleCommandDef { name: "get_config", description: "Get current meter configuration" },
            ModuleCommandDef { name: "probe", description: "Self-test instrument connectivity" },
            ModuleCommandDef { name: "list_active", description: "List currently connected meters" },
        ]
    }

    fn handle_command(
        &mut self,
        cmd: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, CommandError> {
        match cmd {
            "detect" => self.cmd_detect(),
            "connect" => {
                let req: ConnectRequest = serde_json::from_value(payload)?;
                self.cmd_connect(req)
            }
            "disconnect" => {
                let req: DisconnectRequest = serde_json::from_value(payload)?;
                self.cmd_disconnect(req)
            }
            "read" => {
                let req: ReadRequest = serde_json::from_value(payload)?;
                self.cmd_read(req)
            }
            "read_continuous" => {
                let req: ReadContinuousRequest = serde_json::from_value(payload)?;
                self.cmd_read_continuous(req)
            }
            "stop_continuous" => {
                let req: StopContinuousRequest = serde_json::from_value(payload)?;
                self.cmd_stop_continuous(req)
            }
            "set_config" => {
                let req: SetConfigRequest = serde_json::from_value(payload)?;
                self.cmd_set_config(req)
            }
            "get_config" => {
                let req: GetConfigRequest = serde_json::from_value(payload)?;
                self.cmd_get_config(req)
            }
            "probe" => {
                let req: ProbeRequest = serde_json::from_value(payload)?;
                self.cmd_probe(req)
            }
            "list_active" => self.cmd_list_active(),
            _ => Err(CommandError::UnknownCommand(cmd.to_string())),
        }
    }

    fn event_stream(&self) -> Option<tokio::sync::broadcast::Receiver<ModuleEvent>> {
        // Return a receiver for measurement events
        None // populated during initialize
    }
}
```

### 2.4 Command Payload Types

```rust
#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub instrument_id: String,      // from detect() response
    pub config: Option<MeterConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DisconnectRequest {
    pub meter_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ReadRequest {
    pub meter_id: String,
    pub target: Option<TargetColor>,
    pub label: Option<String>,
    pub stabilization_delay_ms: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ReadContinuousRequest {
    pub meter_id: String,
    pub interval_ms: u32,           // minimum time between reads
    pub target: Option<TargetColor>,
}

#[derive(Debug, Deserialize)]
pub struct StopContinuousRequest {
    pub meter_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SetConfigRequest {
    pub meter_id: String,
    pub config: MeterConfig,
}

#[derive(Debug, Deserialize)]
pub struct GetConfigRequest {
    pub meter_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ProbeRequest {
    pub instrument_id: String,
}
```

### 2.5 Detection and USB Enumeration

```rust
// crates/module-meter/src/detection.rs

/// Detect all supported instruments connected via USB.
pub fn detect_instruments() -> Result<Vec<DetectedInstrument>, DetectionError> {
    let mut results = Vec::new();

    // 1. Native HID enumeration (Linux/Windows)
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        results.extend(detect_hid_meters()?);
    }

    // 2. Native USB enumeration (Linux/Windows)
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        results.extend(detect_usb_spectros()?);
    }

    // 3. ArgyllCMS enumeration (all platforms)
    results.extend(detect_argyll_instruments()?);

    // Deduplicate: if the same physical device is found by both native and Argyll,
    // prefer native (marked as `native_available: true`)
    Ok(deduplicate_instruments(results))
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct DetectedInstrument {
    pub id: String,                 // stable ID: "hid:1234:5678:ABCD" or "argyll:1"
    pub model: String,              // "i1 Display Pro Rev.B"
    pub manufacturer: String,       // "X-Rite"
    pub instrument_type: InstrumentType, // Colorimeter, Spectrophotometer
    pub connection_method: ConnectionMethod, // HidNative, UsbNative, ArgyllCMS
    pub serial_number: Option<String>,
    pub usb_vid: Option<u16>,
    pub usb_pid: Option<u16>,
    pub native_driver_available: bool,
    pub argyll_port: Option<u8>,    // ArgyllCMS port number (1, 2, ...)
    pub capabilities: Vec<MeterCapability>,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
pub enum InstrumentType {
    Colorimeter,
    Spectrophotometer,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
pub enum ConnectionMethod {
    HidNative,
    UsbNative,
    ArgyllCMS,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
pub enum MeterCapability {
    EmissiveMeasurement,
    AmbientMeasurement,
    FlashMeasurement,
    SpectralData,       // provides full spectrum, not just XYZ
    HighLuminance,      // >2000 nits (HDR-capable)
}
```

**Deduplication logic:** Two `DetectedInstrument` records are the same physical device if they share `usb_vid`, `usb_pid`, and `serial_number`. In a tie, the native driver record wins; ArgyllCMS record is kept as fallback with `native_driver_available: false`.

### 2.6 Connection and Driver Selection

When the frontend calls `connect(instrument_id)`:

1. Look up `instrument_id` in the detection cache.
2. If `native_driver_available` and platform allows → instantiate native driver (`I1DisplayPro`, `I1Pro2`).
3. Else → instantiate `ArgyllMeter` with the appropriate port.
4. Call `meter.connect()`.
5. If connect succeeds, add to `active_meters` and emit `MeterConnected` event.
6. If connect fails, return error with suggestion (e.g., "ArgyllCMS not installed — run `brew install argyll-cms`").

```rust
impl MeterModule {
    fn create_meter_driver(&self, detected: &DetectedInstrument) -> Result<Box<dyn Meter>, DriverError> {
        match detected.connection_method {
            ConnectionMethod::HidNative if cfg!(any(target_os = "linux", target_os = "windows")) => {
                Ok(Box::new(I1DisplayPro::new(detected.usb_vid.unwrap(), detected.usb_pid.unwrap())?))
            }
            ConnectionMethod::UsbNative if cfg!(any(target_os = "linux", target_os = "windows")) => {
                Ok(Box::new(I1Pro2::new(detected.usb_vid.unwrap(), detected.usb_pid.unwrap())?))
            }
            ConnectionMethod::ArgyllCMS | _ => {
                let port = detected.argyll_port.unwrap_or(1);
                Ok(Box::new(ArgyllMeter::new(port)?))
            }
        }
    }
}
```

### 2.7 ArgyllCMS Integration Strategy

**Why subprocess, not library linking:**
- ArgyllCMS is AGPL. Linking it would force ArtifexProCal to be AGPL.
- Subprocess communication via PTY (pseudo-terminal) is the standard approach used by DisplayCAL and other tools.
- `spotread` provides a stable CLI: `spotread -c <port> -a` (automatic read mode) or interactive XYZ reads.

**V1 to V2 Adaptation:**

The v1 `ArgyllMeter` used `std::process::Command` and blocking `BufReader::read_line`. The v2 adapter uses `tokio::process::Command` and async I/O:

```rust
// crates/module-meter/src/argyll_adapter.rs

pub struct AsyncArgyllMeter {
    port: u8,
    child: Option<tokio::process::Child>,
    reader: Option<tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>>,
}

impl AsyncArgyllMeter {
    pub async fn connect(&mut self) -> Result<(), MeterError> {
        let mut child = tokio::process::Command::new("spotread")
            .arg("-c").arg(self.port.to_string())
            .arg("-a") // automatic read
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| MeterError::ArgyllNotInstalled)?;

        let stdout = child.stdout.take().ok_or(MeterError::NoStdout)?;
        let reader = tokio::io::BufReader::new(stdout).lines();

        self.child = Some(child);
        self.reader = Some(reader);
        Ok(())
    }

    pub async fn read_xyz(&mut self) -> Result<Xyz, MeterError> {
        let reader = self.reader.as_mut().ok_or(MeterError::NotConnected)?;
        let line = tokio::time::timeout(
            Duration::from_secs(30),
            reader.next_line()
        ).await??;

        parse_spotread_output(&line.ok_or(MeterError::NoOutput)?)
    }

    pub async fn disconnect(&mut self) -> Result<(), MeterError> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
        }
        Ok(())
    }

    pub async fn probe(&mut self) -> Result<bool, MeterError> {
        self.connect().await?;
        let _ = self.read_xyz().await?;
        self.disconnect().await?;
        Ok(true)
    }
}
```

**Port mapping:**
- Port 1 = i1 Display Pro (or first colorimeter)
- Port 2 = i1 Pro 2 (or first spectrophotometer)
- Higher ports = additional instruments

The detection routine runs `spotread -?` or `argyll-dispread -?` to list available ports and instrument names.

### 2.8 Native HID Driver Adaptation (i1 Display Pro)

The v1 `I1DisplayPro` driver uses blocking HID I/O. The v2 adapter wraps it in `tokio::task::spawn_blocking`:

```rust
// crates/module-meter/src/hid_adapter.rs

pub struct AsyncI1DisplayPro {
    inner: Arc<Mutex<I1DisplayPro>>,
}

impl AsyncI1DisplayPro {
    pub fn new(vid: u16, pid: u16) -> Result<Self, MeterError> {
        let inner = I1DisplayPro::new(vid, pid)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub async fn connect(&mut self) -> Result<(), MeterError> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = inner.lock().unwrap();
            guard.connect()
        })
        .await
        .map_err(|e| MeterError::JoinError(e.to_string()))?
    }

    pub async fn read_xyz(&mut self) -> Result<Xyz, MeterError> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = inner.lock().unwrap();
            guard.read_xyz()
        })
        .await
        .map_err(|e| MeterError::JoinError(e.to_string()))?
    }

    pub async fn disconnect(&mut self) -> Result<(), MeterError> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = inner.lock().unwrap();
            guard.disconnect()
        })
        .await
        .map_err(|e| MeterError::JoinError(e.to_string()))?
    }

    pub async fn probe(&mut self) -> Result<bool, MeterError> {
        self.connect().await?;
        let _ = self.read_xyz().await?;
        self.disconnect().await?;
        Ok(true)
    }
}
```

**Unlock protocol:**
- 11 known OEM keys exist for factory-locked devices.
- `I1D3_ESCAPE` environment variable allows override for unknown variants.
- Protocol proven correct on physical hardware (v1 archive).

### 2.9 Continuous Measurement

Continuous measurement is implemented as a background tokio task, not a blocking loop:

```rust
impl MeterModule {
    async fn cmd_read_continuous(&mut self, req: ReadContinuousRequest) -> Result<serde_json::Value, CommandError> {
        let meter = self.get_active_meter(&req.meter_id)?;
        let event_bus = self.ctx.as_ref().unwrap().event_bus.clone();
        let meter_id = req.meter_id.clone();
        let interval = Duration::from_millis(req.interval_ms as u64);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                match meter.read_xyz().await {
                    Ok(xyz) => {
                        let result = MeasurementResult::from_xyz(xyz, &meter_id);
                        event_bus.publish(AppEvent::MeasurementProgress(MeasurementProgress {
                            meter_id: meter_id.clone(),
                            result,
                        }));
                    }
                    Err(e) => {
                        event_bus.publish(AppEvent::Error {
                            source: "meter".to_string(),
                            message: e.to_string(),
                        });
                    }
                }
            }
        });

        // Store handle so `stop_continuous` can abort it
        self.continuous_handles.insert(req.meter_id, handle);
        Ok(serde_json::json!({ "status": "started" }))
    }
}
```

**Rule:** Use `tokio::time::interval`, not `std::thread::sleep`. Frontend receives events via Tauri `listen()`.

### 2.10 Correction Matrix Application

The correction matrix is applied at the `Meter` trait level, not in the calibration flow:

```rust
pub struct CorrectingMeter {
    inner: Box<dyn Meter>,
    matrix: [[f64; 3]; 3],
}

impl Meter for CorrectingMeter {
    fn read_xyz(&mut self) -> Result<Xyz, MeterError> {
        let raw = self.inner.read_xyz()?;
        let corrected = apply_matrix(&raw, &self.matrix);
        Ok(corrected)
    }

    // delegate other methods to inner
}
```

When a meter is connected with a correction matrix ID, the `MeterModule` wraps the raw driver in `CorrectingMeter` before adding to `active_meters`.

### 2.11 MeasurementResult Struct (Full Specification)

`MeasurementResult` lives in the `color-science` crate and is the universal data contract for any module that produces colorimetric readings. It is consumed by the workflow engine, storage layer, visualization components, and reporting module.

```rust
/// A single colorimetric measurement from any supported instrument.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MeasurementResult {
    /// UTC timestamp when the measurement completed
    pub timestamp: DateTime<Utc>,

    /// Identifier of the meter module that produced this reading
    pub meter_id: String,

    /// Identifier of the specific instrument (serial number or USB path)
    pub instrument_id: String,

    /// Name of the instrument model (e.g., "i1 Display Pro Rev.B")
    pub instrument_model: String,

    // --- Raw instrument data ---
    /// CIE XYZ tristimulus values (cd/m² for Y)
    pub xyz: Xyz,

    /// Derived CIE xyY chromaticity + luminance
    pub xyy: XyY,

    /// Derived CIE Lab (D65 reference white)
    pub lab: Lab,

    /// Derived CIE LCh (Lightness, Chroma, Hue)
    pub lch: LCh,

    /// ICtCp perceptual color difference space (for HDR)
    pub ictcp: Option<ICtCp>,

    /// Spectral radiance data (only from spectrophotometers like i1 Pro 2)
    pub spectrum: Option<Vec<f64>>,

    /// Spectral wavelengths corresponding to `spectrum` (nm)
    pub spectrum_wavelengths: Option<Vec<f64>>,

    // --- Color Temperature & Quality ---
    /// Correlated Color Temperature (Kelvin)
    pub cct: Option<f64>,

    /// Distance from Planckian locus (Duv)
    pub duv: Option<f64>,

    /// Color Rendering Index (if spectrum available)
    pub cri: Option<f64>,

    // --- Target and error metrics ---
    /// Target color (the intended RGB or xyY of the displayed patch)
    pub target: TargetColor,

    /// DeltaE 2000 against target (if target is known)
    pub delta_e_2000: Option<f64>,

    /// DeltaE ITU-R BT.2124 (if applicable)
    pub delta_e_itp: Option<f64>,

    /// DeltaE 1976 (euclidean in Lab)
    pub delta_e_76: Option<f64>,

    /// DeltaE 1994
    pub delta_e_94: Option<f64>,

    // --- Instrument metadata ---
    /// Integration time in milliseconds
    pub integration_time_ms: Option<u32>,

    /// Whether the instrument reported the reading as saturated
    pub saturated: bool,

    /// Dark current / black level subtraction applied
    pub dark_current_subtracted: bool,

    /// Correction matrix applied (if any)
    pub correction_matrix_applied: Option<String>, // matrix ID or name

    // --- Patch context ---
    /// The RGB stimulus that was displayed (8-bit or 10-bit, depending on generator)
    pub patch_rgb: Rgb<u16>,

    /// Which pattern generator produced the patch
    pub pattern_generator_id: Option<String>,

    /// Which display was being measured
    pub display_id: Option<String>,

    /// Picture mode active on the display during measurement
    pub picture_mode: Option<String>,

    // --- Session context ---
    /// Workflow session this reading belongs to
    pub session_id: Option<String>,

    /// Sequential index within the session
    pub sequence_index: Option<usize>,

    /// User-defined label or tag
    pub label: Option<String>,
}
```

**ArgyllPRO Feature Parity Notes:**
- ArgyllPRO ColorMeter 2 on Android displays: XYZ, Yxy, Lab, LCh, RGB, DeltaE, CCT, Spectrum (for spectros)
- Our `MeasurementResult` adds: ICtCp (for HDR), CRI, Duv, and full patch/session context for calibration workflows
- The `spectral_data` field enables future spectral visualization and CRI calculation without re-measuring

---

## 3. IPC Surface

### 3.1 Commands (Frontend → Backend)

| Command | Request Type | Response Type | Description |
|---------|-------------|---------------|-------------|
| `meter.detect` | `{}` | `Vec<DetectedInstrument>` | Enumerate all connected instruments |
| `meter.connect` | `ConnectRequest` | `MeterConnectionResult` | Open instrument, return meter ID |
| `meter.disconnect` | `DisconnectRequest` | `()` | Close instrument |
| `meter.read` | `ReadRequest` | `MeasurementResult` | Single spot read |
| `meter.read_continuous` | `ReadContinuousRequest` | `{ status: "started" }` | Begin streaming reads |
| `meter.stop_continuous` | `StopContinuousRequest` | `{ status: "stopped" }` | End streaming reads |
| `meter.set_config` | `SetConfigRequest` | `MeterConfig` | Update meter settings |
| `meter.get_config` | `GetConfigRequest` | `MeterConfig` | Read meter settings |
| `meter.probe` | `ProbeRequest` | `{ healthy: bool, message?: string }` | Self-test instrument |
| `meter.list_active` | `{}` | `Vec<ActiveMeterInfo>` | List connected meters |

### 3.2 Events (Backend → Frontend)

| Event | Payload | Description |
|-------|---------|-------------|
| `meter:detected` | `Vec<DetectedInstrument>` | Detection completed (auto-triggered on app startup or manual refresh) |
| `meter:connected` | `ActiveMeterInfo` | Instrument successfully connected |
| `meter:disconnected` | `{ meter_id: string }` | Instrument disconnected |
| `meter:measurement` | `MeasurementResult` | Single measurement completed (from `read` or continuous stream) |
| `meter:health` | `MeterHealth` | Periodic health/status update |
| `meter:error` | `{ meter_id: string, message: string }` | Instrument error |

---

## 4. React Frontend Design

### 4.1 Component Tree

```
MeterModule (registered in ModuleRegistry)
├── MeterSettingsPanel          # Global settings > Meter section
│   ├── DriverPreferenceSelect   # Native / ArgyllCMS / Auto
│   ├── DefaultIntegrationTime  # ms input
│   ├── StabilizationDelay      # ms input
│   └── AutoDarkCurrentToggle   # checkbox
│
├── MeterMonitorPanel           # Live view during workflow or standalone
│   ├── MeterConnectionBar      # dropdown of detected instruments + connect button
│   │   ├── DetectButton
│   │   ├── InstrumentDropdown
│   │   └── ConnectToggle
│   ├── MeasurementDisplay      # large XYZ / xyY / Lab readout
│   │   ├── XyzCard
│   │   ├── XyYCard
│   │   ├── LabCard
│   │   └── DeltaECard (if target provided)
│   ├── MeasurementControls     # read / continuous / stop buttons
│   │   ├── ReadButton
│   │   ├── ContinuousToggle
│   │   └── StabilizationDelayInput
│   ├── MeasurementHistoryTable # scrollable list of recent readings
│   │   └── MeasurementRow
│   └── MeterHealthIndicator    # connection status, read count, error count
│
├── MeterQuickActions           # Dashboard quick actions
│   ├── QuickDetectButton
│   ├── QuickConnectButton
│   └── QuickReadButton
│
└── StandaloneMeterView         # Full-page spot-read mode (no workflow)
    ├── MeterConnectionBar
    ├── MeasurementDisplay
    ├── MeasurementControls
    ├── MeasurementHistoryTable
    ├── TargetSelector          # choose target color for DeltaE calculation
    └── ExportButton            # CSV/JSON export of session readings
```

### 4.2 Standalone Meter Mode

The `StandaloneMeterView` is the primary UI for Phase 1 (no workflow engine yet). It provides:

- **Device connection:** Detect, connect, disconnect, probe
- **Spot reads:** Single measurement with real-time display of XYZ, xyY, Lab, LCh, DeltaE
- **Continuous monitoring:** Streaming reads at configurable intervals
- **Measurement history:** Scrollable table with export to CSV/JSON
- **Target selection:** Choose a target color to compute DeltaE against
- **Export:** Save measurement history for later analysis

This is the first end-to-end feature that proves the module architecture works.

### 4.3 Key Components

#### `MeterConnectionBar`

```typescript
interface MeterConnectionBarProps {
  detectedInstruments: DetectedInstrument[];
  activeMeters: ActiveMeterInfo[];
  onDetect: () => void;
  onConnect: (instrumentId: string, config?: MeterConfig) => void;
  onDisconnect: (meterId: string) => void;
  onProbe: (instrumentId: string) => void;
}
```

- Shows a dropdown of detected instruments with icons (colorimeter vs spectro)
- Connect button opens the instrument; Disconnect closes it
- Probe button runs self-test and shows pass/fail inline
- If no instruments detected, shows helpful message: "No instruments found. Ensure USB cable is connected and drivers are installed."

#### `MeasurementDisplay`

```typescript
interface MeasurementDisplayProps {
  measurement: MeasurementResult | null;
  target?: TargetColor;
  showDeltaE: boolean;
}
```

- Large, high-contrast cards for XYZ, xyY, Lab values
- If `target` is provided and `showDeltaE` is true, shows DeltaE 2000 with color-coded severity:
  - < 1.0: Excellent (green)
  - 1.0–3.0: Good (yellow)
  - > 3.0: Needs work (red)

#### `MeasurementHistoryTable`

```typescript
interface MeasurementHistoryTableProps {
  measurements: MeasurementResult[];
  onSelect: (m: MeasurementResult) => void;
  onExport: (format: "csv" | "json") => void;
}
```

- Sortable columns: timestamp, XYZ, xyY, Lab, DeltaE, label
- Export to CSV or JSON

### 4.4 State Management

```typescript
// src/modules/meter/store.ts

interface MeterStore {
  // Detection state
  detectedInstruments: DetectedInstrument[];
  isDetecting: boolean;
  detect: () => Promise<void>;

  // Active meters
  activeMeters: ActiveMeterInfo[];
  connect: (instrumentId: string) => Promise<void>;
  disconnect: (meterId: string) => Promise<void>;

  // Measurement state
  latestMeasurement: MeasurementResult | null;
  measurementHistory: MeasurementResult[];
  isReading: boolean;
  isContinuous: boolean;
  read: (meterId: string, target?: TargetColor) => Promise<MeasurementResult>;
  startContinuous: (meterId: string, intervalMs: number) => Promise<void>;
  stopContinuous: (meterId: string) => Promise<void>;

  // Config
  moduleConfig: MeterModuleConfig;
  setModuleConfig: (config: Partial<MeterModuleConfig>) => Promise<void>;
}
```

**Hydration:** On mount, the store calls `invoke("module_command", { moduleId: "meter", command: "list_active" })` and `invoke("module_command", { moduleId: "meter", command: "detect" })`.

**Event subscription:** The store subscribes to `meter:connected`, `meter:disconnected`, `meter:measurement`, and `meter:error` via Tauri `listen()`.

---

## 5. Data Flow

### 5.1 Standalone Spot Read

```
[User clicks "Read" in StandaloneMeterView]
  → invoke("module_command", { moduleId: "meter", command: "read", payload: { meterId, target } })
    → MeterModule::handle_command("read")
      → ActiveMeter::read_xyz()
        → [Native HID via spawn_blocking] or [ArgyllCMS via tokio::process]
      → MeasurementResult::from_xyz(xyz)
      → event_bus.publish(AppEvent::MeasurementProgress(...))
    → [Tauri event] meter:measurement
  → MeterStore receives event, updates latestMeasurement + history
  → MeasurementDisplay re-renders with new values
```

### 5.2 Workflow Integration (Future)

During an AutoCal workflow, the workflow engine calls `meter.read` indirectly:

```
[WorkflowEngine: PreMeasurement step]
  → sequencer.next_patch()
    → pattern_gen.display_patch(rgb)
    → sleep(stabilization_delay)
    → invoke("module_command", { moduleId: "meter", command: "read", payload: { meterId, target: TargetColor::Rgb(rgb) } })
      → MeterModule produces MeasurementResult
      → event_bus.publish(AppEvent::MeasurementProgress(...))
    → workflow_engine.append_shared_data("readings", result)
  → [Tauri event] workflow:state_changed
  → WizardShell updates progress bar and step data
```

---

## 6. Testing Strategy

### 6.1 Unit Tests

- `detection.rs`: Mock `hidapi` and `rusb` to test enumeration logic without physical devices
- `argyll_adapter.rs`: Mock PTY to test `spotread` output parsing
- `i1_display_pro.rs`: Mock HID device to test command sequences and XYZ parsing
- `measurement_result.rs`: Test color space conversions and DeltaE calculations

### 6.2 Integration Tests

- `connect_disconnect`: Detect, connect, read, disconnect cycle
- `continuous_read`: Start continuous, receive 5 measurements, stop
- `correction_matrix`: Apply test matrix, verify output is transformed
- `argyll_fallback`: Force ArgyllCMS driver on Linux, verify PTY communication

### 6.3 Hardware Tests

Documented in `tests/hardware/meter_probe.rs`:
- Run `meter.probe()` against physical i1 Display Pro
- Verify XYZ values are in reasonable range for a known test patch
- Run `meter.probe()` against physical i1 Pro 2
- Verify spectrum data is returned (if spectrophotometer)

**Rule:** Every hardware driver must have a `probe()` method. Hardware tests are documented and reproducible, not one-off manual checks.

---

## 7. Error Handling

| Error | Cause | User Message | Recovery |
|-------|-------|------------|----------|
| `MeterNotFound` | Instrument unplugged | "Meter disconnected unexpectedly. Check USB cable." | Re-detect and reconnect |
| `ArgyllNotInstalled` | `spotread` not in PATH | "ArgyllCMS not found. Install with `brew install argyll-cms`." | Install ArgyllCMS |
| `UnlockFailed` | Challenge-response mismatch | "Failed to unlock meter. Try I1D3_ESCAPE env var." | Set env var or use ArgyllCMS |
| `MeasurementTimeout` | Meter unresponsive | "Measurement timed out. Try increasing integration time." | Increase timeout |
| `Saturated` | Reading exceeds meter range | "Signal too bright. Use ND filter or reduce display brightness." | Adjust display |
| `CalibrationRequired` | Meter needs calibration | "Meter requires calibration. Place on calibration tile and retry." | Calibrate meter |

---

## 8. Open Questions

1. **Multi-meter support:** Should the UI allow two meters connected simultaneously (e.g., spectro for profiling, colorimeter for speed)? Recommendation: yes — `active_meters` is a `Vec`, and the UI shows a tab per meter.
2. **ArgyllCMS path customization:** Should users specify a custom `spotread` path? Recommendation: yes, via `SettingsStore` key `"meter.argyll_path"`; default is `"spotread"` in PATH.
3. **Spectral data visualization:** The i1 Pro 2 returns spectral data. Should the MeterModule emit a separate `spectral_data` event, or include it in `MeasurementResult.spectrum`? Recommendation: include in `MeasurementResult`; the frontend decides whether to render a spectral plot.
4. **Dark current storage:** Should dark current readings be stored in SQLite for drift tracking? Recommendation: yes, in `meter_initializations` table with timestamp and dark current XYZ.

---

## 9. Approval Checklist

Before implementation begins, confirm:
- [ ] Instrument list (§1.1) covers all devices the user owns or plans to support
- [ ] Platform routing (§1.2) is correct for the user's development environment (macOS primary)
- [ ] `CalibrationModule` trait implementation (§2.3) is sufficient for MeterModule needs
- [ ] Command payload types (§2.4) cover all frontend use cases
- [ ] ArgyllCMS subprocess strategy (§2.7) is acceptable (AGPL-safe)
- [ ] Native HID async adapter (§2.8) correctly wraps v1 blocking I/O
- [ ] Continuous measurement (§2.9) uses non-blocking tokio tasks
- [ ] Correction matrix is applied at HAL level (§2.10), not in workflow
- [ ] `MeasurementResult` (§2.11) has all fields needed for ArgyllPRO parity
- [ ] React component tree (§4.1) covers standalone mode + workflow integration
- [ ] Testing strategy (§6) includes hardware `probe()` tests
