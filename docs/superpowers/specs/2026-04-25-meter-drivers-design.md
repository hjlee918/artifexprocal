# Phase 3c: Meter Drivers Design Spec

> **Date:** 2026-04-25
> **Scope:** X-Rite i1 Display Pro Rev.B and i1 Pro 2 HID drivers with initialization tracking
> **Approach:** `hidapi` crate, `hal_meters` crate, SQLite-backed init tracker

---

## 1. Goal

Implement real HID-based meter drivers for the two primary color measurement devices in the calibration loop:
- **i1 Display Pro Rev.B** — Colorimeter (2000 nits HDR capable), fast, requires meter profiling for OLED
- **i1 Pro 2** — Spectrophotometer, accurate reference for profiling, requires 3-hour white patch initialization

Both implement the existing `hal::traits::Meter` trait so they drop into `GreyscaleAutoCalFlow` without changes.

---

## 2. Architecture

### New Crate

| Crate | Responsibility |
|-------|---------------|
| `hal_meters` | HID enumeration, X-Rite command protocol, meter drivers |

### Crate Dependencies

```
hal_meters
├── hal (traits + types + errors)
├── color_science (XYZ, RGB types)
├── calibration-storage (SQLite init tracker)
├── hidapi (HID device access)
├── chrono (timestamps)
├── serde (serialization)
└── thiserror (error types)
```

### File Structure

```
crates/hal_meters/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── hid_util.rs          # Shared: VID/PID lookup, HID enumeration
    ├── commands.rs          # X-Rite HID command constants
    ├── i1_display_pro.rs    # i1 Display Pro Rev.B driver
    ├── i1_pro_2.rs          # i1 Pro 2 driver
    ├── spectro_trait.rs       # Spectrophotometer extension trait
    ├── init_tracker.rs        # SQLite-backed last-init timestamp
    └── profiling.rs           # Meter correction matrix generation
```

---

## 3. Shared HID Infrastructure

### VID/PID Registry

```rust
pub struct XriteDevice {
    pub vid: u16,
    pub pid: u16,
    pub name: &'static str,
}

pub const I1_DISPLAY_PRO: XriteDevice = XriteDevice { vid: 0x0765, pid: 0x5020, name: "i1 Display Pro Rev.B" };
pub const I1_PRO_2: XriteDevice = XriteDevice { vid: 0x0765, pid: 0x5034, name: "i1 Pro 2" };
```

### HID Utility Functions

```rust
pub fn enumerate_xrite_devices() -> Vec<(HidDeviceInfo, XriteDevice)>;
pub fn open_device(xrite: &XriteDevice) -> Result<HidDevice, HidError>;
pub fn send_command(device: &mut HidDevice, cmd: &[u8]) -> Result<(), HidError>;
pub fn read_response(device: &mut HidDevice, timeout_ms: i32) -> Result<Vec<u8>, HidError>;
```

---

## 4. X-Rite HID Protocol

### Command Constants

```rust
pub const CMD_GET_FIRMWARE: u8 = 0x01;
pub const CMD_SET_EMISSIVE: u8 = 0x02;
pub const CMD_TRIGGER_MEASURE: u8 = 0x03;
pub const CMD_READ_XYZ: u8 = 0x04;
pub const CMD_READ_SPECTRUM: u8 = 0x05;
pub const CMD_INITIALIZE: u8 = 0x06;
pub const CMD_SET_INTEGRATION_TIME: u8 = 0x07;
```

### Response Format

All responses are 64-byte HID reports:

| Offset | Length | Description |
|--------|--------|-------------|
| 0 | 1 | Status byte (0x00 = OK, 0xFF = error) |
| 1 | 1 | Command echo |
| 2–5 | 4 | X as IEEE 754 float32 (cd/m² scaled) |
| 6–9 | 4 | Y as float32 |
| 10–13 | 4 | Z as float32 |
| 14+ | — | Extended data (spectrum, firmware string, etc.) |

### Status Codes

```rust
pub enum XriteStatus {
    Ok = 0x00,
    Busy = 0x01,
    Error = 0xFF,
    InitializationRequired = 0xFE,
}
```

---

## 5. i1 Display Pro Rev.B

### Implementation

```rust
pub struct I1DisplayPro {
    device: Option<HidDevice>,
    info: XriteDevice,
    serial: Option<String>,
    integration_time_ms: u32,
}

impl I1DisplayPro {
    pub fn new() -> Self { /* find and cache device info */ }
    pub fn set_integration_time(&mut self, ms: u32);
}

impl Meter for I1DisplayPro {
    fn connect(&mut self) -> Result<(), MeterError>;
    fn disconnect(&mut self);
    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError>;
    fn model(&self) -> &str { "i1 Display Pro Rev.B" }
}
```

### Connect Flow

1. Enumerate HID devices matching VID/PID
2. Open first matching device
3. Send `CMD_GET_FIRMWARE` — verify response
4. Send `CMD_SET_EMISSIVE` — configure for emissive display measurement
5. Cache serial number from device info

### Read XYZ Flow

1. If `integration_time_ms` differs from cached, send `CMD_SET_INTEGRATION_TIME`
2. Send `CMD_TRIGGER_MEASURE`
3. Read 64-byte response
4. Parse X, Y, Z from float32 at offsets 2, 6, 10
5. Scale from meter units to cd/m² (Y is luminance)

### HDR Mode

The i1 Display Pro Rev.B supports up to 2000 nits. For HDR measurements, use longer integration times (500–2000ms) to improve signal-to-noise ratio at low light levels.

---

## 6. i1 Pro 2

### Implementation

```rust
pub struct I1Pro2 {
    device: Option<HidDevice>,
    info: XriteDevice,
    serial: Option<String>,
    init_tracker: Option<InitTracker>,
}

impl I1Pro2 {
    pub fn new() -> Self;
    pub fn initialize(&mut self) -> Result<(), MeterDriverError>;
    pub fn read_spectrum(&mut self) -> Result<Spectrum, MeterDriverError>;
    pub fn time_until_init_expires(&self) -> Option<Duration>;
}

impl Meter for I1Pro2 {
    fn connect(&mut self) -> Result<(), MeterError>;
    fn disconnect(&mut self);
    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError>;
    fn model(&self) -> &str { "i1 Pro 2" }
}
```

### Connect Flow

1. Enumerate HID devices matching VID/PID
2. Open first matching device
3. Send `CMD_GET_FIRMWARE`
4. Check initialization status — if expired, return `MeterError::InitializationRequired`
5. Optionally: attach `InitTracker` for timestamp persistence

### Initialize Flow

1. Display prompt to user: "Place i1 Pro 2 on white patch and press OK"
2. Send `CMD_INITIALIZE`
3. Wait for response (status 0x00)
4. Record timestamp in `InitTracker`

### Read Spectrum

1. Send `CMD_TRIGGER_MEASURE`
2. Read response
3. Parse 36 spectral values (380–730nm, 10nm steps) as float32 array from offset 14
4. Return `Spectrum` struct

### Initialization Timer

- **Requirement:** Must re-initialize every 3 hours with a white patch
- **Tracking:** SQLite table `meter_initializations` stores last-init timestamp per serial
- **UI:** Backend exposes `time_until_init_expires()`; frontend shows countdown timer
- **Behavior:** `connect()` returns `InitializationRequired` if expired; user must call `initialize()`

---

## 7. Spectrophotometer Extension Trait

The i1 Pro 2 exposes spectral data. This is the foundation for meter profiling.

```rust
pub trait Spectrophotometer: Meter {
    fn read_spectrum(&mut self) -> Result<Spectrum, MeterError>;
    fn wavelengths() -> &'static [f64] {
        // 380, 390, ..., 730
        &[
            380.0, 390.0, 400.0, 410.0, 420.0, 430.0, 440.0, 450.0,
            460.0, 470.0, 480.0, 490.0, 500.0, 510.0, 520.0, 530.0,
            540.0, 550.0, 560.0, 570.0, 580.0, 590.0, 600.0, 610.0,
            620.0, 630.0, 640.0, 650.0, 660.0, 670.0, 680.0, 690.0,
            700.0, 710.0, 720.0, 730.0,
        ]
    }
}

pub struct Spectrum {
    pub values: [f64; 36], // 380-730nm, 10nm steps
}
```

---

## 8. Init Tracker (SQLite)

### Schema

```sql
CREATE TABLE IF NOT EXISTS meter_initializations (
    meter_serial TEXT PRIMARY KEY,
    meter_model TEXT NOT NULL,
    last_init_at TEXT NOT NULL,  -- ISO 8601
    expires_at TEXT NOT NULL     -- last_init_at + 3 hours
);
```

### API

```rust
pub struct InitTracker {
    conn: rusqlite::Connection,
}

impl InitTracker {
    pub fn new(conn: &rusqlite::Connection) -> Self;
    pub fn record_init(&self, serial: &str, model: &str) -> Result<(), InitTrackerError>;
    pub fn time_until_next_init(&self, serial: &str) -> Option<Duration>;
    pub fn is_init_expired(&self, serial: &str) -> bool;
}
```

### Behavior

- On `record_init`: Insert or replace row with current timestamp + 3h expiry
- On `time_until_next_init`: Query `expires_at`, compute `Duration` from now
- On `is_init_expired`: Query `expires_at`, compare to `Utc::now()`
- If no record exists for serial: returns `None` (treat as expired)

---

## 9. Meter Profiling (Foundation)

Meter profiling creates a correction matrix for a colorimeter against a spectrophotometer reference.

### Profiling Flow

```rust
pub fn generate_correction_matrix(
    spectro: &mut dyn Spectrophotometer,
    colorimeter: &mut dyn Meter,
    patches: &[RGB],
) -> Result<CorrectionMatrix, ProfilingError> {
    // 1. Measure each patch with both meters
    // 2. Compute XYZ from spectro spectrum
    // 3. Build linear system: colorimeter_XYZ * M ≈ spectro_XYZ
    // 4. Solve 3x3 matrix M via least squares
}
```

### CorrectionMatrix

```rust
pub struct CorrectionMatrix {
    pub matrix: [[f64; 3]; 3], // 3x3 correction matrix
}

impl CorrectionMatrix {
    pub fn apply(&self, xyz: &XYZ) -> XYZ {
        XYZ {
            x: self.matrix[0][0] * xyz.x + self.matrix[0][1] * xyz.y + self.matrix[0][2] * xyz.z,
            y: self.matrix[1][0] * xyz.x + self.matrix[1][1] * xyz.y + self.matrix[1][2] * xyz.z,
            z: self.matrix[2][0] * xyz.x + self.matrix[2][1] * xyz.y + self.matrix[2][2] * xyz.z,
        }
    }
}
```

### Notes

- Full profiling implementation is out of scope for this phase — foundation only
- `generate_correction_matrix` is stubbed with mock data for now
- Real profiling requires 24+ patch measurements (grayscale + primaries + secondaries)

---

## 10. Error Handling

### MeterDriverError

```rust
#[derive(Debug, thiserror::Error)]
pub enum MeterDriverError {
    #[error("HID error: {0}")]
    HidError(String),
    #[error("Device not found: {name} (VID {vid:04X}, PID {pid:04X})")]
    DeviceNotFound { name: String, vid: u16, pid: u16 },
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Initialization required for {meter}")]
    InitializationRequired { meter: String },
    #[error("Read timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u32 },
    #[error("Init tracker error: {0}")]
    InitTrackerError(String),
}
```

Maps to `hal::error::MeterError` for trait compatibility.

---

## 11. HAL Trait Integration

Both meters implement `hal::traits::Meter`:

```rust
impl Meter for I1DisplayPro {
    fn connect(&mut self) -> Result<(), MeterError> { ... }
    fn disconnect(&mut self) { ... }
    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError> { ... }
    fn model(&self) -> &str { "i1 Display Pro Rev.B" }
}

impl Meter for I1Pro2 {
    fn connect(&mut self) -> Result<(), MeterError> { ... }
    fn disconnect(&mut self) { ... }
    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError> { ... }
    fn model(&self) -> &str { "i1 Pro 2" }
}
```

---

## 12. Testing Strategy

### Unit Tests (mock HID)

- `MockHidDevice` simulates 64-byte responses
- Test connect flow, XYZ parsing, status code handling
- Test init tracker SQLite operations

### Hardware Tests (`#[ignore]`)

- `test_real_i1_display_pro_connect`: Enumerate, connect, disconnect
- `test_real_i1_display_pro_read`: Read XYZ, verify values are reasonable (Y > 0)
- `test_real_i1_pro_2_connect`: Enumerate, connect, disconnect
- `test_real_i1_pro_2_read`: Read XYZ + spectrum
- `test_real_i1_pro_2_initialize`: Full init flow with white patch

### Integration Test

```rust
#[test]
fn test_both_meters_read_same_patch() {
    // Connect both meters
    // Display white patch via PGenerator or iTPG
    // Read XYZ from both
    // Assert DeltaE between readings < 5.0 (loose tolerance for unprofiled colorimeter)
}
```

---

## 13. UI Integration (Future Phase)

Settings panel will expose:

```
Meters:
  i1 Display Pro Rev.B
    [Connect] [Disconnect]
    Integration time: [200] ms

  i1 Pro 2
    [Connect] [Disconnect]
    [Initialize] — Next init required in: 2:34:12
    [Profile Colorimeter] — Generate correction matrix
```

The init countdown timer queries `InitTracker::time_until_next_init()` via Tauri command.

---

## 14. Spec Self-Review

### Placeholder Scan
- No TBD/TODO placeholders. All endpoints, commands, and types are defined.

### Internal Consistency
- `Meter` trait unchanged — both drivers implement it directly
- `InitTracker` uses `calibration-storage` crate for SQLite (existing dependency)
- `hidapi` is the only new external dependency
- `Spectrophotometer` trait is additive — does not break existing consumers

### Scope Check
- Focused on driver implementation + init tracking
- Profiling is foundation-only (matrix type + stubbed function)
- UI integration noted as future work

### Ambiguity Check
- 3-hour init rule is explicit
- `read_xyz` returns `XYZ` in cd/m² (Y is luminance)
- Spectrum is 36 values, 380–730nm, 10nm steps
- HID report size is 64 bytes (standard for X-Rite)

---

## Appendix: X-Rite HID Protocol Reference

### i1 Display Pro Rev.B

| Command | Code | Payload | Response |
|---------|------|---------|----------|
| Get Firmware | 0x01 | — | Firmware string |
| Set Emissive | 0x02 | — | Status |
| Trigger Measure | 0x03 | — | Status |
| Read XYZ | 0x04 | — | XYZ float32 |
| Set Integration Time | 0x07 | u32 ms | Status |

### i1 Pro 2

| Command | Code | Payload | Response |
|---------|------|---------|----------|
| Get Firmware | 0x01 | — | Firmware string |
| Set Emissive | 0x02 | — | Status |
| Trigger Measure | 0x03 | — | Status |
| Read XYZ | 0x04 | — | XYZ float32 |
| Read Spectrum | 0x05 | — | 36x float32 |
| Initialize | 0x06 | — | Status |
| Set Integration Time | 0x07 | u32 ms | Status |

---

## Appendix: Competitor Reference

- **ArgyllCMS** (`spotread`) — open-source reference for X-Rite HID protocol
- **ColourSpace** — uses same HID commands, proprietary interpretation
- **DisplayCAL** — Python wrapper around ArgyllCMS for X-Rite meters
