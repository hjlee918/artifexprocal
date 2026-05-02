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
    pub registers: HashMap<RegisterSlot, MeasurementResult>,
    pub preset: MeterPreset,
    pub configurable_readouts: Vec<ReadoutField>, // which MeasurementResult fields appear in MeasurementDisplay
    pub palette_storage_path: String,
    pub ccss_install_path: String,
    pub ccmx_install_path: String,
    pub oem_file_install_path: String,
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
    Emissive,     // Display direct measurement (default)
    Ambient,      // Ambient light measurement
    Flash,        // Flash / projector measurement
    Telephoto,    // Measurement through lens
    Reflective,   // Reflective surface measurement (print, paper, textiles)
    Transmissive, // Transmissive measurement (film, backlit displays, filters)
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

#[derive(Debug, Deserialize)]
pub struct SetRegisterRequest {
    pub meter_id: String,
    pub register_slot: RegisterSlot,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClearRegisterRequest {
    pub meter_id: String,
    pub register_slot: RegisterSlot,
}

#[derive(Debug, Deserialize)]
pub struct SwapRegistersRequest {
    pub meter_id: String,
    pub slot_a: RegisterSlot,
    pub slot_b: RegisterSlot,
}

#[derive(Debug, Deserialize)]
pub struct RenameRegisterRequest {
    pub meter_id: String,
    pub register_slot: RegisterSlot,
    pub new_label: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum RegisterSlot {
    Current, Reference, W, K, R, G, B, C, M, Y,
}

#[derive(Debug, Deserialize)]
pub struct MatchNamedColorRequest {
    pub meter_id: String,
    pub palette_id: Option<String>, // None = search all palettes
}

#[derive(Debug, Deserialize)]
pub struct ImportPaletteRequest {
    pub file_path: String,
    pub format: PaletteFormat, // Cxf, Icc
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum PaletteFormat {
    Cxf,
    Icc,
}

#[derive(Debug, Deserialize)]
pub struct SetPresetRequest {
    pub meter_id: String,
    pub preset: MeterPreset,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum MeterPreset {
    Printing,
    Photography,
    Lighting,
    GraphicDesign,
    TvVideo,
}

#[derive(Debug, Deserialize)]
pub struct ExportMeasurementsRequest {
    pub meter_id: String,
    pub file_path: String,
    pub format: ExportFormat,
    pub filter: MeasurementFilter,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum ExportFormat {
    Tsv,
    Csv,
    Json,
    ArgyllSp,
}

#[derive(Debug, Deserialize, Default)]
pub struct MeasurementFilter {
    pub mode: Option<MeasurementMode>,
    pub instrument_id: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub register: Option<RegisterSlot>,
    pub session_id: Option<String>,
    pub search_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetTpgPatchRequest {
    pub meter_id: String,
    pub patch_rgb: Rgb<u16>,
    pub colorspace: RgbSpace,
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
    ReflectiveMeasurement,
    TransmissiveMeasurement,
    SpectralData,       // provides full spectrum, not just XYZ
    HighLuminance,      // >2000 nits (HDR-capable)
    DensityMeasurement, // Status A/M/T/E, ISO Type 1/2, Visual
    LuxMeasurement,     // incident illuminance (photography mode)
    RefreshRateDetection, // display refresh rate detection
    UvMeasurement,      // UV index / ARPANSA exposure
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

    /// Derived CIE Yu'v' (uniform chromaticity scale)
    pub yuv_uv_prime: Option<XyY>,

    /// Derived CIE L*u*v* (CIELUV)
    pub luv: Option<Lab>,

    /// Derived CIE L*C*h*uv (polar form of CIELUV)
    pub lchuv: Option<LCh>,

    /// Derived DIN99 Lab (perceptually uniform alternative to CIELAB)
    pub din99_lab: Option<Lab>,

    /// ICtCp perceptual color difference space (for HDR)
    pub ictcp: Option<ICtCp>,

    /// Spectral radiance data (only from spectrophotometers like i1 Pro 2)
    pub spectrum: Option<Vec<f64>>,

    /// Spectral wavelengths corresponding to `spectrum` (nm)
    pub spectrum_wavelengths: Option<Vec<f64>>,

    /// CIE standard observer used for tristimulus calculation from spectrum
    pub observer_type: Option<ObserverType>,

    /// Selected illuminant for reflective/transmissive calculations
    pub illuminant_selection: Option<IlluminantSelection>,

    // --- Color Temperature & Quality ---
    /// Correlated Color Temperature (Kelvin)
    pub cct: Option<f64>,

    /// Distance from Planckian locus (Duv)
    pub duv: Option<f64>,

    /// Delta CCT vs. reference register (K)
    pub delta_cct: Option<f64>,

    /// Black Body or Daylight locus mode used for CCT calculation
    pub locus_mode: Option<LocusMode>,

    /// CCT calculation method (Correlated or DE2000)
    pub cct_method: Option<CctMethod>,

    /// Color temperature in Mired (1,000,000 / CCT)
    pub mired: Option<f64>,

    /// Color rendering metrics (if spectrum available)
    pub color_rendering: Option<ColorRendering>,

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

    /// DeltaE 1994 Textile variant
    pub delta_e_94_textile: Option<f64>,

    /// DeltaE CMC 2:1 (acceptability)
    pub delta_e_cmc_2_1: Option<f64>,

    /// DeltaE CMC 1:1 (perceptibility)
    pub delta_e_cmc_1_1: Option<f64>,

    /// DeltaE DIN99
    pub delta_e_din99: Option<f64>,

    // --- Instrument metadata ---
    /// Integration time in milliseconds
    pub integration_time_ms: Option<u32>,

    /// Whether the instrument reported the reading as saturated
    pub saturated: bool,

    /// Dark current / black level subtraction applied
    pub dark_current_subtracted: bool,

    /// Correction matrix applied (if any)
    pub correction_matrix_applied: Option<String>, // matrix ID or name

    /// Density measurements (only in Reflective / Transmissive mode)
    pub density: Option<DensityValues>,

    /// Incident illuminance in lux (photography / lighting mode)
    pub illuminance_lux: Option<f64>,

    /// Reflected luminance in cd/m² (explicit photography field)
    pub reflected_luminance: Option<f64>,

    /// Exposure Value (EV) calculated from illuminance / luminance
    pub ev: Option<f64>,

    /// Visual contrast ratio (luminance_current / luminance_reference)
    pub visual_contrast_ratio: Option<f64>,

    /// UV index calculated from spectral data (ARPANSA method)
    pub uv_index: Option<f64>,

    /// Detected display refresh rate in Hz (if instrument supports it)
    pub detected_refresh_rate_hz: Option<f64>,

    // --- Patch context ---
    /// The RGB stimulus that was displayed (8-bit or 10-bit, depending on generator)
    pub patch_rgb: Rgb<u16>,

    /// Colorspace tag for the RGB stimulus (BT.709, BT.2020, DCI-P3, custom)
    pub colorspace_tag: Option<RgbSpace>,

    /// BT.1886 EOTF response value for the current patch (normalized 0–1)
    pub bt1886_response: Option<f64>,

    /// Delta RGB (measured vs. target per-channel difference)
    pub delta_rgb: Option<Rgb<f64>>,

    /// Which RGB channel(s) need adjustment to reach target (per-channel direction)
    pub adjustment_direction_rgb: Option<Rgb<AdjustmentDirection>>,

    /// Which CMY channel(s) need adjustment (inverse of RGB for display calibration)
    pub adjustment_direction_cmy: Option<Rgb<AdjustmentDirection>>,

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

/// Color rendering metrics derived from spectral data.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ColorRendering {
    /// CIE 1995 General Color Rendering Index (Ra)
    pub cri_ra: f64,
    /// CIE R9 (saturated red) — reported separately per ArgyllPRO convention
    pub cri_r9: f64,
    /// EBU TLCI-2012 Qa (Television Lighting Consistency Index)
    pub tlci_qa: Option<f64>,
    /// TM-30-15 metrics (if spectrum available)
    pub tm_30_15: Option<Tm30Metrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Tm30Metrics {
    /// Fidelity index (Rf)
    pub rf: f64,
    /// Gamut index (Rg)
    pub rg: f64,
}

/// Standard CIE observers for tristimulus calculation from spectral data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum ObserverType {
    Cie1931_2deg,
    Cie1964_10deg,
}

/// Standard illuminants for reflective/transmissive calculations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum IlluminantSelection {
    D50,
    D55,
    D65,
    D75,
    A,   // Incandescent
    E,   // Equal energy
    Custom { spectrum_id: String },
}

/// Black body or daylight locus mode for CCT calculation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum LocusMode {
    BlackBody,
    Daylight,
}

/// CCT calculation method.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum CctMethod {
    Correlated,
    De2000,
}

/// Per-channel adjustment direction for RGB/CMY display calibration guidance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
pub enum AdjustmentDirection {
    Increase,
    Decrease,
    Neutral,
}

/// Density measurement values for print and film workflows.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DensityValues {
    /// Visual density (unfiltered)
    pub visual: f64,
    /// Status A (offset print, tungsten light)
    pub status_a: Option<f64>,
    /// Status M (reflection densitometry)
    pub status_m: Option<f64>,
    /// Status T (transmission densitometry, wideband)
    pub status_t: Option<f64>,
    /// Status E (DIN narrowband)
    pub status_e: Option<f64>,
    /// ISO Type 1 (graphic arts, narrowband)
    pub iso_type_1: Option<f64>,
    /// ISO Type 2 (graphic arts, wideband)
    pub iso_type_2: Option<f64>,
}

**ArgyllPRO Feature Parity Notes:**
- ArgyllPRO ColorMeter 2 on Android displays: XYZ, Yxy, Lab, LCh, RGB, DeltaE, CCT, Spectrum (for spectros)
- Our `MeasurementResult` adds: ICtCp (for HDR), full color rendering struct (CRI Ra+R9, TLCI, TM-30-15), Duv, density, photography fields, and full patch/session context for calibration workflows
- The `spectral_data` field enables future spectral visualization and CRI calculation without re-measuring

---

## 2.12 Density Measurements

Density measurements are print-workflow specific and only applicable in **Reflective** or **Transmissive** measurement modes. The meter module computes density from the measured luminance (or transmittance) relative to a calibrated white (or clear) reference.

**Supported density types:**

| Type | Filter | Use Case |
|------|--------|----------|
| Visual | None (unfiltered) | General-purpose density |
| Status A | ISO 5-3 A | Offset print, tungsten light |
| Status M | ISO 5-3 M | Reflection densitometry |
| Status T | ISO 5-3 T | Transmission densitometry (wideband) |
| Status E (DIN) | DIN narrowband | European standard |
| ISO Type 1 | Narrowband | Graphic arts, narrow spectral |
| ISO Type 2 | Wideband | Graphic arts, wide spectral |

**Workflow:**
1. User selects Reflective or Transmissive mode.
2. Meter takes a baseline reading on unprinted paper (or clear film) — stored as the **Reference** register.
3. Subsequent readings compute `density = -log10(measured / reference)`.
4. Results populate `MeasurementResult.density`.

**UI:** A `DensityPanel` component appears in `StandaloneMeterView` when a meter supports `DensityMeasurement`. It shows all seven density values in a grid. The baseline-clear button resets the reference.

---

## 2.13 Photography Mode Details

Photography mode provides lighting and exposure metrics used by photographers, cinematographers, and gaffers.

**Fields added to `MeasurementResult`:**
- `illuminance_lux` — incident illuminance (lux) when meter is in Ambient or Flash mode with cosine diffuser
- `reflected_luminance` — reflected luminance in cd/m² (explicit field, not derived from XYZ Y)
- `ev` — Exposure Value calculated from illuminance or luminance using standard photographic equations

**Exposure Value calculation:**
```
EV = log2(lux / 2.5)      // for incident light
EV = log2(luminance * ISO / K)  // for reflected light (K = 12.5 for standard meters)
```

**Interactive Exposure Calculator (`ExposureCalculator` component):**
- Inputs: ISO, aperture (f-stop), shutter speed, EV (from meter)
- Outputs: the missing variable (e.g., given ISO + aperture + EV, compute required shutter speed)
- Displayed as a card in `StandaloneMeterView` when `MeasurementMode` is Ambient or Flash.
- Supports bulb mode, fractional stops, and ND filter compensation.

---

## 2.14 Color Library System

The Color Library subsystem allows users to compare measurements against named colors, palettes, and industry standards.

**Features:**

1. **Web RGB Display** — Every `MeasurementResult` computes an sRGB hex value (`#RRGGBB`) for quick visual reference. Displayed in `MeasurementDisplay` as a small color swatch.

2. **Visual Compare Swatches** — Side-by-side swatch comparison of Current measurement vs. Reference register, or vs. a named color from the library.

3. **Named Color Matching** — Find the closest named color to the current measurement using DeltaE 2000. Supports:
   - CSS / X11 named colors (147 colors)
   - Pantone (if palette imported)
   - NCS (Natural Color System, if imported)
   - User-defined custom named colors

4. **Named Color Visual Swatch (`NamedColorSwatch` component)** — A small card showing the matched named color name, hex value, and DeltaE distance.

5. **Palette Import:**
   - **CxF (.cxf)** — Color eXchange Format palette import. Parsed in Rust, stored in SQLite `palettes` table.
   - **ICC / ICM palette (.icc, .icm)** — Extracts named color tags from ICC v4 profiles.

6. **Storage:** Palettes live in `calibration-storage` (`palettes` table). Each palette has a name, source file path, and color rows.

7. **Search / Filter UI (`PaletteBrowser` component):** Search by color name, filter by palette, sort by hue / lightness / chroma.

**IPC additions:**
- `meter.match_named_color` → returns closest named color + DeltaE
- `meter.list_palettes` → returns all installed palettes
- `meter.import_palette` → accepts file path + format, returns palette ID

---

## 2.15 Measurement Registers

Registers are persistent named measurement slots that enable comparison, reference tracking, and multi-point analysis.

**Register slots:**

| Slot | Purpose | Default Label |
|------|---------|---------------|
| Current | The most recent measurement | "Current" |
| Reference | The reference against which DeltaE is computed | "Reference" |
| W | White point measurement | "White" |
| K | Black point measurement | "Black" |
| R | Red primary measurement | "Red" |
| G | Green primary measurement | "Green" |
| B | Blue primary measurement | "Blue" |
| C | Cyan secondary measurement | "Cyan" |
| M | Magenta secondary measurement | "Magenta" |
| Y | Yellow secondary measurement | "Yellow" |

**Behavior:**
- Every new measurement automatically populates the **Current** register.
- **Reference** drives `DeltaE` calculations in `MeasurementDisplay`. The user can set any register as Reference.
- Registers can be renamed, cleared, or swapped.
- All registers are stored in `MeterModuleConfig` and persisted to `SettingsStore`.

**IPC commands:**
- `meter.set_register` — Store current measurement into a named slot
- `meter.clear_register` — Remove a stored measurement
- `meter.swap_registers` — Exchange two register slots
- `meter.rename_register` — Change the display label of a slot
- `meter.get_all_registers` — Return all populated registers

**UI (`RegisterManager` component):**
- Grid of 10 register cards showing color swatch, Lab values, and label.
- Click to set as Reference, rename, or clear.
- "Set from Current" button on each empty slot.
- Displayed in `StandaloneMeterView` as an accordion panel.

---

## 2.16 Specialty Measurements

Specialty measurements cover niche use cases and advanced instrument capabilities.

**Visual Contrast:**
- Computes luminance ratio between Current and Reference registers: `contrast_ratio = max(L_current, L_reference) / min(L_current, L_reference)`.
- Displayed as a ratio (e.g., "21:1") and as a percentage.
- Useful for accessibility compliance (WCAG) and display uniformity checks.

**ARPANSA UV Exposure:**
- Calculates UV index from spectral data using the ARPANSA erythemal action spectrum weighting.
- Only available when the instrument returns full spectral data (`spectrum` is `Some`).
- Displayed in `MeasurementDisplay` as a small UV index badge when applicable.

**Display Refresh Rate Detection:**
- Some meters (e.g., Klein K-10A) can detect the refresh rate of the display being measured.
- Populates `MeasurementResult.detected_refresh_rate_hz`.
- Shown in `MeterHealthIndicator` as an extra status line when available.

---

## 2.17 Chromaticity Display

The Chromaticity Display subsystem renders CIE diagrams with live measurement plotting, reference overlays, and gamut triangles.

**Supported diagrams:**
- **CIE 1931 xy** — Standard chromaticity diagram
- **CIE 1976 u'v'** — Uniform chromaticity scale (recommended for small color differences)

**Features:**
- **Live point plotting** — Current measurement appears as a pulsating dot that updates on every read.
- **Reference point overlay** — The Reference register is shown as a fixed crosshair.
- **Register points overlay** — All populated registers (W, K, R, G, B, C, M, Y) are shown as labeled dots.
- **Gamut triangle overlay** — BT.709, BT.2020, and DCI-P3 triangles can be toggled on/off.
- **dE vector** — When Current and Reference are both set, an arrow shows the direction and magnitude of the color difference.

**React component: `ChromaticityDiagram`**
- Rendering: **SVG-based** (per Lesson #5: Three.js is NOT permitted until Phase 5+).
- Uses pre-computed spectral locus path data for CIE 1931 and 1976.
- Responsive, zoomable, with axis labels and gridlines.
- Located in `StandaloneMeterView` as the primary visualization panel.

**Data contract:**
```typescript
interface ChromaticityDiagramProps {
  diagramType: 'cie1931_xy' | 'cie1976_uv';
  currentPoint: { x: number; y: number } | null;
  referencePoint: { x: number; y: number } | null;
  registerPoints: Array<{ label: string; x: number; y: number; color: string }>;
  showGamutTriangles: boolean[];
}
```

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
| `meter.set_register` | `SetRegisterRequest` | `RegisterState` | Store current measurement into a register slot |
| `meter.clear_register` | `ClearRegisterRequest` | `()` | Clear a register slot |
| `meter.swap_registers` | `SwapRegistersRequest` | `()` | Swap two register slots |
| `meter.rename_register` | `RenameRegisterRequest` | `RegisterState` | Rename a register slot label |
| `meter.get_all_registers` | `{}` | `Vec<RegisterState>` | Return all populated registers |
| `meter.match_named_color` | `MatchNamedColorRequest` | `NamedColorMatchResult` | Find closest named color to current measurement |
| `meter.list_palettes` | `{}` | `Vec<PaletteSummary>` | List installed color palettes |
| `meter.import_palette` | `ImportPaletteRequest` | `PaletteSummary` | Import a CxF or ICC palette file |
| `meter.set_preset` | `SetPresetRequest` | `MeterModuleConfig` | Apply a preset configuration |
| `meter.get_presets` | `{}` | `Vec<PresetSummary>` | List available presets |
| `meter.export_measurements` | `ExportMeasurementsRequest` | `{ file_path: string, format: string }` | Export filtered measurements (TSV, CSV, JSON, .sp) |
| `meter.set_tpg_patch` | `SetTpgPatchRequest` | `()` | Send live patch color to pattern generator |

### 3.1.1 Phase 1 Export Schema

This section defines the exact CSV and JSON export schemas shipped in Phase 1. Future phases may extend these schemas; the `schemaVersion` field identifies the format revision.

#### CSV Export (`export.csv`)

**RFC 4180 compliant.** Header row is mandatory. Missing numeric values are represented as empty fields (`,,`). Free-text fields (label, instrument_model) are double-quoted if they contain commas or newlines. `""` is used to escape literal quotes per RFC 4180.

**Column order (fixed, 34 columns):**

<!-- Count must match the enumerated field list below. Update both together when columns change. -->

```
measurement_uuid,schema_version,software_version,timestamp,mode,instrument_model,instrument_id,
x,y,z,xy_x,xy_y,lab_l,lab_a,lab_b,lch_l,lch_c,lch_h,uvp_u,uvp_v,
cct,duv,delta_e_2000,target_x,target_y,
patch_r,patch_g,patch_b,patch_bit_depth,patch_colorspace,reference_white,
session_id,sequence_index,label
```

**Field definitions:**

| # | Field | Type | Description / Constraints |
|---|-------|------|---------------------------|
| 1 | `measurement_uuid` | UUID v4 (RFC 4122) | Generated per measurement; stable across re-exports. UUID is assigned at measurement time and persisted on the `MeasurementResult` struct; re-exporting the same measurement always produces the same UUID. |
| 2 | `schema_version` | literal string | `"1.0"` for Phase 1 |
| 3 | `software_version` | string | Application version string, e.g. `"2.0.0-phase1"` |
| 4 | `timestamp` | ISO 8601 | Millisecond precision, Z suffix: `2026-04-30T12:00:00.123Z` |
| 5 | `mode` | string | `Emissive` or `Ambient` |
| 6 | `instrument_model` | string | e.g. `FakeMeter`, `i1 Display Pro Rev.B` |
| 7 | `instrument_id` | string | Serial number, USB path, or mock ID |
| 8–10 | `x`, `y`, `z` | f64 | CIE XYZ tristimulus values. `y` is luminance in cd/m² (CIE Y = luminance for emissive sources). |
| 11–12 | `xy_x`, `xy_y` | f64 | CIE 1931 xy chromaticity coordinates |
| 13–15 | `lab_l`, `lab_a`, `lab_b` | f64 | CIELAB D65¹ |
| 16–18 | `lch_l`, `lch_c`, `lch_h` | f64 | CIE LCh (Lightness, Chroma, Hue in degrees) |
| 19–20 | `uvp_u`, `uvp_v` | f64 | CIE 1976 u′v′ (UCS) chromaticity |
| 21 | `cct` | f64 | Correlated color temperature in Kelvin |
| 22 | `duv` | f64 | Distance from Planckian locus. Positive Duv = green side of the locus (higher v′ in CIE 1976 UCS); negative = magenta side, per Ohno 2013. |
| 23 | `delta_e_2000` | f64 | vs. target; empty if no target set |
| 24–25 | `target_x`, `target_y` | f64 | Target xy chromaticity (if set); empty otherwise |
| 26–28 | `patch_r`, `patch_g`, `patch_b` | u16 | RGB stimulus in **16-bit full range** (0–65535). Conversion from source bit depth: `out = (raw * 65535) / (2^bit_depth − 1)` |
| 29 | `patch_bit_depth` | u8 | Source bit depth: `8`, `10`, `12`, or `16` |
| 30 | `patch_colorspace` | string | `BT.709`, `BT.2020`, `DCI-P3`, `Display-P3`, `sRGB`, `AdobeRGB`, `ProPhoto`, or empty |
| 31 | `reference_white` | string | Reference white point: `"D65"` in Phase 1; extensible for D50/D55/etc. |
| 32 | `session_id` | string | Workflow session UUID; empty in standalone mode |
| 33 | `sequence_index` | usize | Zero-based position within session; empty if not in a session |
| 34 | `label` | string | User-defined label; empty if unset |

¹ Lab uses D65 reference white Xn = 95.047, Yn = 100.000, Zn = 108.883 (CIE 2° standard observer). |

**Bit-depth conversion rule:**
When the source patch data uses a bit depth other than 16, the export normalizes to 16-bit full range using the formula:
```
normalized = round(raw_value * 65535.0 / (2^bit_depth - 1))
```
For example, a 10-bit value of `512` becomes `round(512 * 65535 / 1023) = 32768`.

#### JSON Export (`export.json`)

Array of objects. Keys are camelCase to match TypeScript conventions. All numeric fields are JSON numbers (f64). `null` is used for optional fields (target, sessionId) to distinguish "unset" from "zero."

```typescript
// src/types/export.ts — single source of truth
// Rust equivalent is auto-generated via ts-rs or specta

export interface Phase1MeasurementExport {
  measurementUuid: string;        // UUID v4
  schemaVersion: "1.0";
  softwareVersion: string;
  timestamp: string;            // ISO 8601 with ms + Z
  mode: "Emissive" | "Ambient";
  instrument: {
    model: string;
    id: string;
  };
  xyz: { x: number; y: number; z: number };
  xyy: { x: number; y: number; yLum: number };
  lab: { l: number; a: number; b: number };
  lch: { l: number; c: number; h: number };
  uvPrime: { u: number; v: number };
  cct: number;
  duv: number;
  deltaE2000: number | null;
  target: { x: number; y: number } | null;
  patchRgb: { r: number; g: number; b: number }; // 16-bit normalized
  patchBitDepth: 8 | 10 | 12 | 16;
  patchColorspace:
    | "BT.709" | "BT.2020" | "DCI-P3" | "Display-P3"
    | "sRGB" | "AdobeRGB" | "ProPhoto"
    | "";
  referenceWhite: "D65" | "D50" | "D55" | "D75" | "C" | "E"; // Phase 1 emits "D65"
  sessionId: string | null;
  sequenceIndex: number | null;
  label: string;
}
```

**Schema integrity rule:**
The TypeScript interface above is the single source of truth. Rust types are generated from it via `ts-rs` (or `specta` if already integrated). A `schema.json` (JSON Schema draft 2020-12) is committed alongside the code at `docs/schemas/meter-export-phase1.json` and validated in CI.

#### Example CSV Row (single measurement, 80% gray patch)

```csv
measurement_uuid,schema_version,software_version,timestamp,mode,instrument_model,instrument_id,x,y,z,xy_x,xy_y,lab_l,lab_a,lab_b,lch_l,lch_c,lch_h,uvp_u,uvp_v,cct,duv,delta_e_2000,target_x,target_y,patch_r,patch_g,patch_b,patch_bit_depth,patch_colorspace,reference_white,session_id,sequence_index,label
550e8400-e29b-41d4-a716-446655440000,1.0,2.0.0-phase1,2026-04-30T12:00:00.123Z,Emissive,FakeMeter,mock:planckian-42,76.037,80.0,87.106,0.3127,0.3290,83.138,0.0,-1.803,80.0,1.803,270.0,0.1978,0.4683,6504.0,0.0,,,,52428,52428,52428,16,BT.709,D65,,0,"80% gray"
```

#### Example JSON Object (same measurement)

```json
{
  "measurementUuid": "550e8400-e29b-41d4-a716-446655440000",
  "schemaVersion": "1.0",
  "softwareVersion": "2.0.0-phase1",
  "timestamp": "2026-04-30T12:00:00.123Z",
  "mode": "Emissive",
  "instrument": { "model": "FakeMeter", "id": "mock:planckian-42" },
  "xyz": { "x": 76.037, "y": 80.0, "z": 87.106 },
  "xyy": { "x": 0.3127, "y": 0.3290, "yLum": 80.0 },
  "lab": { "l": 83.138, "a": 0.0, "b": -1.803 },
  "lch": { "l": 83.138, "c": 1.803, "h": 270.0 },
  "uvPrime": { "u": 0.1978, "v": 0.4683 },
  "cct": 6504.0,
  "duv": 0.0,
  "deltaE2000": null,
  "target": null,
  "patchRgb": { "r": 52428, "g": 52428, "b": 52428 },
  "patchBitDepth": 16,
  "patchColorspace": "BT.709",
  "referenceWhite": "D65",
  "sessionId": null,
  "sequenceIndex": null,
  "label": "80% gray"
}
```

#### CI Validation

A test in the `module-meter` crate generates a sample `Phase1MeasurementExport`, serializes it to JSON, and validates it against `docs/schemas/meter-export-phase1.json` using the `jsonschema` crate (or equivalent). The test fails the build on schema mismatch. A corresponding CSV test generates a sample row, parses it back with `csv` crate, and asserts each typed field matches expected values. Both tests run in CI on every commit.

### 3.2 Events (Backend → Frontend)

| Event | Payload | Description |
|-------|---------|-------------|
| `meter:detected` | `Vec<DetectedInstrument>` | Detection completed (auto-triggered on app startup or manual refresh) |
| `meter:connected` | `ActiveMeterInfo` | Instrument successfully connected |
| `meter:disconnected` | `{ meter_id: string }` | Instrument disconnected |
| `meter:measurement` | `MeasurementResult` | Single measurement completed (from `read` or continuous stream) |
| `meter:health` | `MeterHealth` | Periodic health/status update |
| `meter:error` | `{ meter_id: string, message: string }` | Instrument error |
| `meter:register_changed` | `RegisterState` | A register slot was updated |
| `meter:palette_imported` | `PaletteSummary` | A palette was successfully imported |
| `meter:chromaticity_update` | `ChromaticityPoint` | New point for chromaticity diagram (throttled) |
| `meter:tpg_patch_set` | `{ r: u8, g: u8, b: u8 }` | Confirmation that pattern generator patch was updated |

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
    ├── ChromaticityDiagram     # SVG-based CIE 1931 / 1976 live plot
    ├── MeasurementDisplay
    │   ├── XyzCard
    │   ├── XyYCard
    │   ├── LabCard
    │   ├── DeltaECard (if target provided)
    │   ├── ColorSwatch         # sRGB hex swatch of current measurement
    │   ├── NamedColorSwatch    # closest named color match
    │   ├── DensityPanel        # visible when in Reflective/Transmissive mode
    │   └── ExposureCalculator  # visible when in Ambient/Flash mode
    ├── MeasurementControls
    ├── MeasurementHistoryTable
    ├── RegisterManager         # 10-slot register grid
    ├── TargetSelector          # choose target color for DeltaE calculation
    ├── PaletteBrowser          # search/filter named colors
    ├── DensityBaselinePanel    # set/clear density reference
    └── ExportButton            # TSV/CSV/JSON/.sp export of session readings
```

### 4.2 Standalone Meter Mode

The `StandaloneMeterView` is the primary UI for Phase 1 (no workflow engine yet). It provides:

- **Device connection:** Detect, connect, disconnect, probe
- **Spot reads:** Single measurement with real-time display of XYZ, xyY, Lab, LCh, DeltaE
- **Continuous monitoring:** Streaming reads at configurable intervals
- **Measurement history:** Scrollable table with export to TSV/CSV/JSON/.sp
- **Target selection:** Choose a target color to compute DeltaE against
- **Register management:** 10 named slots (Current, Reference, W, K, R, G, B, C, M, Y) for comparison and reference tracking
- **Chromaticity display:** SVG-based CIE 1931 xy or 1976 u'v' diagram with live plotting and gamut triangles
- **Color library:** Named color matching, palette import (CxF / ICC), visual swatch comparison
- **Density panel:** Print-workflow density values (Visual, Status A/M/T/E, ISO Type 1/2) in Reflective/Transmissive mode
- **Exposure calculator:** Photography mode with lux, EV, and interactive exposure calculator
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
- Shows sRGB hex `ColorSwatch` for quick visual reference
- `NamedColorSwatch` displays the closest named color match and DeltaE distance
- `DensityPanel` appears when `measurement_mode` is Reflective or Transmissive
- `ExposureCalculator` appears when `measurement_mode` is Ambient or Flash
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
- Filter by mode, instrument, date range, register, session
- Search across labels, notes, and session IDs
- Export to TSV, CSV, JSON, or ArgyllCMS .sp format (bulk or per-measurement)

#### `ChromaticityDiagram`

```typescript
interface ChromaticityDiagramProps {
  diagramType: 'cie1931_xy' | 'cie1976_uv';
  currentPoint: { x: number; y: number } | null;
  referencePoint: { x: number; y: number } | null;
  registerPoints: Array<{ label: string; x: number; y: number; color: string }>;
  showGamutTriangles: boolean[];
}
```

- SVG-based rendering (no Three.js per Lesson #5)
- Live pulsating dot for current measurement
- Fixed crosshair for Reference register
- Labeled dots for all populated registers (W, K, R, G, B, C, M, Y)
- Toggle-able gamut triangles: BT.709, BT.2020, DCI-P3
- dE vector arrow when Current and Reference are both set

#### `RegisterManager`

```typescript
interface RegisterManagerProps {
  registers: RegisterState[];
  onSetReference: (slot: RegisterSlot) => void;
  onClear: (slot: RegisterSlot) => void;
  onSwap: (slotA: RegisterSlot, slotB: RegisterSlot) => void;
  onRename: (slot: RegisterSlot, label: string) => void;
  onSetFromCurrent: (slot: RegisterSlot) => void;
}
```

- 10-slot grid with color swatch, Lab values, and editable label
- Click any slot to set it as the Reference register
- "Set from Current" button on empty slots
- Accordion panel in `StandaloneMeterView`

#### `DensityPanel`

```typescript
interface DensityPanelProps {
  density: DensityValues | null;
  baselineSet: boolean;
  onSetBaseline: () => void;
  onClearBaseline: () => void;
}
```

- Grid of 7 density values: Visual, Status A, Status M, Status T, Status E, ISO Type 1, ISO Type 2
- Baseline reference controls (set/clear)
- Only visible when meter is in Reflective or Transmissive mode

#### `ExposureCalculator`

```typescript
interface ExposureCalculatorProps {
  ev: number | null;
  iso: number;
  onIsoChange: (iso: number) => void;
}
```

- Interactive solver: given ISO + aperture + EV → shutter speed, or any permutation
- Supports bulb mode, fractional stops, ND filter compensation
- Only visible when meter is in Ambient or Flash mode

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

  // Registers
  registers: RegisterState[];
  setRegister: (slot: RegisterSlot, label?: string) => Promise<void>;
  clearRegister: (slot: RegisterSlot) => Promise<void>;
  swapRegisters: (slotA: RegisterSlot, slotB: RegisterSlot) => Promise<void>;
  renameRegister: (slot: RegisterSlot, label: string) => Promise<void>;

  // Color library
  palettes: PaletteSummary[];
  importPalette: (filePath: string, format: PaletteFormat) => Promise<void>;
  matchNamedColor: (paletteId?: string) => Promise<NamedColorMatchResult | null>;

  // Presets
  presets: PresetSummary[];
  applyPreset: (preset: MeterPreset) => Promise<void>;

  // Config
  moduleConfig: MeterModuleConfig;
  setModuleConfig: (config: Partial<MeterModuleConfig>) => Promise<void>;

  // Export
  exportMeasurements: (filePath: string, format: ExportFormat, filter?: MeasurementFilter) => Promise<void>;
}
```

**Hydration:** On mount, the store calls `invoke("module_command", { moduleId: "meter", command: "list_active" })` and `invoke("module_command", { moduleId: "meter", command: "detect" })`.

**Event subscription:** The store subscribes to `meter:connected`, `meter:disconnected`, `meter:measurement`, `meter:error`, `meter:register_changed`, `meter:palette_imported`, `meter:chromaticity_update`, and `meter:tpg_patch_set` via Tauri `listen()`.

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
| `DensityBaselineMissing` | Density computed before baseline set | "Set density baseline on unprinted paper before measuring." | Set baseline |
| `PaletteImportFailed` | Invalid CxF or ICC file | "Failed to import palette. Check file format and try again." | Re-export palette |
| `NamedColorMatchFailed` | No palettes installed | "Install a color palette before matching named colors." | Import palette |
| `ExportFormatUnsupported` | Requested export format not implemented | "Export format not yet supported." | Try TSV or CSV |
| `RegisterSlotEmpty` | Operation on empty register | "Register slot is empty. Store a measurement first." | Set register |
| `TpgPatchFailed` | Pattern generator did not acknowledge patch | "Pattern generator did not update. Check connection." | Reconnect TPG |
| `PresetApplyFailed` | Meter does not support preset capabilities | "Preset requires capabilities this meter does not have." | Choose different preset |

---

## 8. Configuration & Persistence Details

### 8.1 Preset Configurations

Presets configure the MeterModule for common workflows with a single click. Each preset sets measurement mode, enabled readouts, and default filters.

| Preset | Mode | Enabled Readouts | Notes |
|--------|------|-----------------|-------|
| **Printing** | Reflective | Density (all), Lab, DeltaE | Status A default density filter |
| **Photography** | Ambient | Lux, EV, CCT, DeltaE | Enables ExposureCalculator |
| **Lighting** | Ambient | Lux, CCT, Duv, CRI Ra+R9, TLCI | Full color quality metrics |
| **Graphic Design** | Emissive | Lab, LCh, DeltaE, RGB swatch | sRGB-focused readouts |
| **TV & Video** | Emissive | XYZ, xyY, DeltaE 2000, ICtCp, RGB | BT.709/BT.2020 tags enabled |

### 8.2 Configurable Readouts

Users select which `MeasurementResult` fields appear simultaneously in `MeasurementDisplay`. Default readouts per preset are listed above. Custom readout selection is persisted per user.

### 8.3 File Installation Paths

| File Type | Path | Purpose |
|-----------|------|---------|
| `.ccss` | `~/.artifexprocal/spectral/ccss/` | Display calibration spectral sample sets |
| `.ccmx` | `~/.artifexprocal/matrices/ccmx/` | Meter correction matrices |
| `.oem` | `~/.artifexprocal/instruments/oem/` | OEM instrument support files (unlock keys, firmware) |

**Management UI:** A `FileInstallationPanel` in `MeterSettingsPanel` lists installed files by type, allows import via file picker, and validates file format on upload.

### 8.4 Measurement Logging Details

- **Capacity:** 10,000 measurements (rolling buffer in SQLite, paginated).
- **Per-measurement metadata:** timestamp, measurement mode, instrument model/serial, location (lat/lng if available), optional photo blob (for print-workflow traceability).
- **Filter UI:** Filter by mode, instrument, date range, register slot, session ID.
- **Search UI:** Full-text search across labels, notes, and session IDs.

---

## 9. Open Questions

1. **Multi-meter support:** Should the UI allow two meters connected simultaneously (e.g., spectro for profiling, colorimeter for speed)? Recommendation: yes — `active_meters` is a `Vec`, and the UI shows a tab per meter.
2. **ArgyllCMS path customization:** Should users specify a custom `spotread` path? Recommendation: yes, via `SettingsStore` key `"meter.argyll_path"`; default is `"spotread"` in PATH.
3. **Spectral data visualization:** The i1 Pro 2 returns spectral data. Should the MeterModule emit a separate `spectral_data` event, or include it in `MeasurementResult.spectrum`? Recommendation: include in `MeasurementResult`; the frontend decides whether to render a spectral plot.
4. **Dark current storage:** Should dark current readings be stored in SQLite for drift tracking? Recommendation: yes, in `meter_initializations` table with timestamp and dark current XYZ.

---

## 10. Approval Checklist

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

---

## 11. Phase Classification Matrix

Every feature in the MeterModule is classified below as **Phase 1** (first release), **Phase 2** (next iteration), or **Deferred** (with reason).

### 10.1 Measurement Modes

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Emissive | **Phase 1** | Core display calibration mode; primary use case. |
| Ambient | **Phase 1** | Required for photography/lighting workflows; trivial to add via mode enum. |
| Reflective | **Phase 2** | Print workflow; needs density subsystem and illuminant selection. |
| Transmissive | **Phase 2** | Film/transparency workflow; same dependencies as Reflective. |
| Flash | **Phase 2** | Projector/flash photography; needs integration time handling changes. |
| Telephoto | **Phase 2** | Niche; adds no new data types, just UI labeling. |

### 10.2 Tristimulus Colorspaces

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| CIE XYZ | **Phase 1** | Raw instrument output; essential. |
| Yxy | **Phase 1** | Derived from XYZ; single line of code. |
| Lab | **Phase 1** | Industry standard; already in v1 color-science. |
| LCh (ab) | **Phase 1** | Polar form of Lab; trivial derivation. |
| Yu'v' | **Phase 2** | Uniform chromaticity; useful but not critical for MVP. |
| L*u*v* | **Phase 2** | Alternative uniform space; adds no new workflow. |
| L*C*h*uv | **Phase 2** | Polar LUV; same rationale as L*u*v*. |
| DIN99 Lab | **Phase 2** | Perceptually uniform alternative; nice-to-have. |
| ICtCp | **Phase 1** | HDR metric; already planned for BT.2020/PQ workflows. |

### 10.3 Delta E Variants

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| CIE DE2000 | **Phase 1** | Industry standard; primary metric for display calibration. |
| CIE DE76 | **Phase 1** | Simple euclidean fallback; already in v1. |
| CIE DE94 | **Phase 2** | Used in some legacy workflows; one function call. |
| DE94 Textile | **Phase 2** | Niche textile workflow; same formula with different weighting. |
| CMC 2:1 | **Phase 2** | Print industry acceptability metric; low effort. |
| CMC 1:1 | **Phase 2** | Print industry perceptibility metric; low effort. |
| DIN99 DE | **Phase 2** | Only needed if DIN99 Lab is adopted. |
| DeltaE ITU-R BT.2124 | **Phase 1** | HDR metric; already planned. |

### 10.4 Density Measurements

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Visual density | **Phase 2** | Requires Reflective mode + baseline reference; print workflow. |
| Status A / M / T / E | **Phase 2** | Filtered density variants; same block as Visual. |
| ISO Type 1 / 2 | **Phase 2** | Graphic arts density; same block as Visual. |

### 10.5 Photography Mode

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Reflected luminance | **Phase 1** | Already derived from XYZ Y; just adds explicit field. |
| Incident illuminance (lux) | **Phase 2** | Requires cosine diffuser support; Ambient mode enhancement. |
| Exposure Value (EV) | **Phase 2** | Simple calculation from lux/luminance; tied to lux field. |
| Interactive Exposure Calculator | **Deferred** | Complex UI with 3-variable solver; depends on Photography workflow (Phase 3+). |

### 10.6 Color Temperature

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| CCT | **Phase 1** | Essential for white balance evaluation. |
| Duv | **Phase 1** | Trivial add-on to CCT calculation. |
| Delta CT | **Phase 2** | Delta vs. reference; requires register system. |
| Black Body / Daylight locus | **Phase 2** | CCT accuracy improvement; minor math change. |
| Correlated / DE2000 method | **Phase 2** | Method toggle; low effort. |
| Kelvin / Mired units | **Phase 2** | Display toggle; trivial conversion. |

### 10.7 RGB / Video

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| BT.709 / BT.2020 / DCI-P3 tags | **Phase 1** | Enum already exists; just add field to MeasurementResult. |
| Custom RGB space | **Phase 2** | Requires user-defined matrix input. |
| BT.1886 response | **Phase 2** | EOTF calculation; useful for gamma verification. |
| Delta RGB | **Phase 2** | Per-channel difference vs. target. |
| RGB / CMY Adjustment Direction | **Phase 2** | UI guidance arrows; dependent on Delta RGB. |

### 10.8 Color Rendering

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| CRI Ra | **Phase 2** | Requires spectral data; only spectrophotometers provide this. |
| CRI R9 | **Phase 2** | Same block as Ra; single additional spectral integral. |
| TLCI Qa | **Deferred** | Requires EBU test spectrum and detailed spectral math; lighting workflow only. |
| TM-30-15 (Rf, Rg) | **Deferred** | Requires 99-color test sample set and extensive spectral math; lighting workflow only. |

### 10.9 Specialty Measurements

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Visual Contrast | **Phase 2** | Simple ratio of two registers; low effort. |
| ARPANSA UV Exposure | **Deferred** | Niche; requires UV-weighted spectral integral and safety thresholds. |
| Display Refresh Rate detection | **Deferred** | Requires instrument support (Klein K-10A); no current hardware in user's lab. |

### 10.10 Color Library

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Web RGB / sRGB hex swatch | **Phase 1** | Trivial XYZ→sRGB conversion; high visual value. |
| Visual Compare Swatches | **Phase 2** | Side-by-side comparison; requires register system. |
| Named Color matching (CSS/X11) | **Phase 2** | 147-color lookup table; simple DeltaE search. |
| Named Color Visual Swatch | **Phase 2** | UI card; tied to named color matching. |
| CxF palette import | **Phase 2** | XML parsing; useful for Pantone workflows. |
| ICC palette import | **Phase 2** | ICC tag extraction; similar effort to CxF. |

### 10.11 Spectral Display

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| SPD graph | **Phase 2** | SVG line chart; data already in MeasurementResult. |
| Observer selection (1931 2° / 1964 10°) | **Phase 2** | Enum exists; affects tristimulus math from spectrum. |
| Illuminant selection | **Phase 2** | Enum exists; affects reflective/transmissive calculations. |

### 10.12 Chromaticity Display

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| CIE 1931 xy diagram | **Phase 1** | SVG-based; primary visualization for standalone mode. |
| CIE 1976 u'v' diagram | **Phase 2** | Same SVG engine, different locus path; low effort. |
| Reference point overlay | **Phase 1** | Essential for DeltaE visualization. |
| Register points overlay | **Phase 2** | Requires register system. |
| Gamut triangle overlay | **Phase 1** | Static SVG paths; high educational value. |
| dE vector arrow | **Phase 2** | SVG line + arrowhead; requires Current + Reference. |

### 10.13 Measurement Registers

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Current register | **Phase 1** | Implicitly exists (latest measurement). |
| Reference register | **Phase 1** | Drives DeltaE; essential for any comparison workflow. |
| W (White) register | **Phase 1** | Used for white balance and density baseline. |
| K, R, G, B, C, M, Y registers | **Phase 2** | Useful for gamut / CMS workflows but not needed for basic spot reads. |
| Register management UI | **Phase 1** | Reference + W are needed immediately; UI scales to 10 slots. |

### 10.14 Configuration & Persistence

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Save/restore meter configs | **Phase 1** | Already in MeterModuleConfig + SettingsStore. |
| Save/restore instrument configs | **Phase 1** | Serial-specific settings persisted in DeviceInventory. |
| Presets (Printing / Photography / Lighting / Graphic Design / TV & Video) | **Phase 2** | Convenient but not blocking; can default to TV & Video. |
| Configurable readouts | **Phase 2** | UI polish; default readouts cover 90% of use cases. |
| .ccss / .ccmx / OEM file install | **Phase 2** | File management UI; data paths already defined. |

### 10.15 Measurement Logging

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| 1,000 measurement capacity | **Phase 1** | SQLite-backed; sufficient for initial testing and small sessions. |
| 10,000 measurement capacity | **Phase 2** | Pagination and indexing; scaling concern, not correctness. |
| Per-measurement metadata | **Phase 2** | Adds lat/lng and photo blob; not essential for display calibration. |
| Historical review UI | **Phase 1** | MeasurementHistoryTable already planned. |
| Filter and search | **Phase 2** | SQL query enhancement; low effort. |

### 10.16 Export Formats

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| TSV export | **Phase 1** | Simple tab-separated format; one-line formatter change from CSV. |
| CSV export | **Phase 1** | Already planned. |
| JSON export | **Phase 1** | Already planned. |
| ArgyllCMS .sp format | **Phase 2** | Requires spectral wavelength header formatting; spectro-only. |
| Per-measurement export | **Phase 2** | UI selection mode; low effort. |
| Bulk export | **Phase 1** | MeasurementHistoryTable export already covers this. |

### 10.17 Test Pattern Generator Integration

| Feature | Phase | Reasoning |
|---------|-------|-----------|
| Live patch update API (`setPatchColor`) | **Phase 2** | Requires PatternModule to exist; MeterModule can emit event but PatternModule must consume it. |
| Continuous measurement + live patch sync | **Phase 2** | Depends on live patch update + workflow engine timing. |
| Six standard test cards | **Deferred** | Display Geometry, Probe Locations, Low/High Clipping, Neutrals, Color Ramps — these are display-diagnostic patterns that require a mature DisplayModule and pattern sequencing engine (Phase 5+). |

### 10.18 Summary by Phase

| Phase | Feature Count | Key Deliverables |
|-------|--------------|----------------|
| **Phase 1** | ~35 features | Emissive/Ambient modes, XYZ/xyY/Lab/LCh/ICtCp, DE2000/DE76, CCT/Duv, CIE 1931 xy diagram, Current/Reference/W registers, TSV/CSV/JSON export, 1,000-log capacity, sRGB swatch, standalone spot-read UI |
| **Phase 2** | ~45 features | Reflective/Transmissive/Flash, remaining colorspaces, all DeltaE variants, density, photography (lux/EV), full register set, named colors, palette import, spectral observer/illuminant, CIE 1976 u'v', presets, configurable readouts, .ccss/.ccmx install, 10,000-log capacity, .sp export, live TPG patch update |
| **Deferred** | ~6 features | Interactive Exposure Calculator, TLCI, TM-30-15, ARPANSA UV, refresh rate detection, six standard test cards |

**ArgyllPRO Coverage:** 100% of categories are now documented. Every feature is either covered in Phase 1/2 or explicitly Deferred with a documented reason. The Phase 1 minimum delivers a functional, competitive standalone meter application. Phase 2 closes all remaining gaps for professional print, photography, and lighting workflows.
