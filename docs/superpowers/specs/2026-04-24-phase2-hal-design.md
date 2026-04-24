# Phase 2: Hardware Abstraction Layer (HAL) Design

**Date:** 2026-04-24
**Status:** Approved for implementation
**Scope:** HAL traits + mocks + skeleton real implementations

---

## 1. Goal

Build a trait-based Hardware Abstraction Layer that decouples the calibration engine from specific hardware implementations, enabling:
- Mock-based testing of the calibration pipeline
- Skeleton real device stubs for iterative protocol development
- Plugin architecture for future device support

## 2. Architecture

### Single Crate (`crates/hal`)

```
crates/hal/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports
    ├── error.rs            # Error types
    ├── traits.rs           # Meter, DisplayController, PatternGenerator
    ├── mocks.rs            # FakeMeter, FakeDisplay, FakePatternGen
    └── devices/
        ├── mod.rs
        ├── xrite_i1_display_pro.rs
        ├── xrite_i1_pro_2.rs
        ├── lg_oled.rs
        ├── sony_projector.rs
        ├── pgenerator.rs
        └── lg_internal.rs
```

### Rationale for single crate
- Trait surface is still stabilizing; splitting too early creates dependency churn
- Mocks co-located with traits let downstream crates test without extra dev-dependencies
- Device modules are stubs (no real I/O), so compile time is minimal

## 3. Traits

### `Meter` trait
```rust
pub trait Meter: Send + Sync {
    fn connect(&mut self) -> Result<(), MeterError>;
    fn disconnect(&mut self);
    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError>;
    fn model(&self) -> &str;
}
```

### `DisplayController` trait
```rust
pub trait DisplayController: Send + Sync {
    fn connect(&mut self) -> Result<(), DisplayError>;
    fn disconnect(&mut self);
    fn set_picture_mode(&mut self, mode: &str) -> Result<(), DisplayError>;
    fn upload_1d_lut(&mut self, lut: &Lut1D) -> Result<(), DisplayError>;
    fn upload_3d_lut(&mut self, lut: &Lut3D) -> Result<(), DisplayError>;
    fn set_white_balance(&mut self, gains: RGBGain) -> Result<(), DisplayError>;
}
```

### `PatternGenerator` trait
```rust
pub trait PatternGenerator: Send + Sync {
    fn connect(&mut self) -> Result<(), PatternGenError>;
    fn disconnect(&mut self);
    fn display_patch(&mut self, color: &RGB) -> Result<(), PatternGenError>;
}
```

## 4. Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MeterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Read timeout")]
    ReadTimeout,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DisplayError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Upload failed: {0}")]
    UploadFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PatternGenError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Display error: {0}")]
    DisplayError(String),
}
```

## 5. Mock Implementations

### `FakeMeter`
- Returns configurable XYZ values from a preset table
- Simulates integration time delay
- Configurable noise model (optional)

### `FakeDisplayController`
- Records every call for verification
- Stores uploaded LUTs in memory
- Simulates display response time

### `FakePatternGenerator`
- Simulates patch timing
- Configurable settle time before "displaying"
- Records patch history

## 6. Skeleton Real Implementations

### LG OLED (`lg_oled.rs`)
- Config struct with IP, port, pairing code
- `connect()` validates IP format, marks paired state
- Stub methods return `Ok(())` (no real HTTP yet)
- Device-specific types: `LGOLEDModel`, `PictureMode`

### X-Rite i1 Display Pro (`xrite_i1_display_pro.rs`)
- Config struct with HID path (optional)
- `connect()` validates HID path exists
- Stub `read_xyz()` returns fixed test value
- Model string: "i1 Display Pro Rev.B"

### X-Rite i1 Pro 2 (`xrite_i1_pro_2.rs`)
- Config with USB path / Argyll driver path
- `connect()` validates path
- Stub methods for spectral/colorimetric modes

### Sony Projector (`sony_projector.rs`)
- Config with IP/port or serial path
- `connect()` validates connection params
- Stub commands for picture mode, lens memory

### PGenerator (`pgenerator.rs`)
- Config with Raspberry Pi IP
- `connect()` validates IP format
- Stub `display_patch()` with timing simulation

### LG Internal Pattern Generator (`lg_internal.rs`)
- Config with TV IP
- `connect()` reuses LG OLED connection (shared HTTP client)
- Stub patch display over network

## 7. Types

```rust
pub struct Lut1D {
    pub channels: [Vec<f64>; 3], // R, G, B
    pub size: usize,
}

pub struct Lut3D {
    pub data: Vec<RGB>,
    pub size: usize,
}

pub struct RGBGain {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

pub enum PictureMode {
    Standard,
    Cinema,
    Game,
    ExpertDark,
    ExpertBright,
    Custom(String),
}
```

## 8. Testing Strategy

1. **Trait compilation tests** — verify `dyn Meter`, `dyn DisplayController` compile
2. **Mock end-to-end** — `FakeMeter` reads known values → `FakeDisplay` records corrections
3. **Config validation tests** — each device validates IP format, path existence
4. **Error handling tests** — verify error types propagate correctly

## 9. Dependencies

```toml
[dependencies]
color-science = { path = "../color-science" }
thiserror = "1"
```

## 10. Integration Points

- `color-science` crate provides XYZ, RGB, Lab types used in trait signatures
- Future `calibration` crate will consume HAL traits and use mocks for testing
- Tauri IPC will wrap HAL operations in commands

---

## Spec Self-Review

1. **Placeholder scan:** No TBD, TODO, or vague requirements found.
2. **Internal consistency:** Trait signatures match error types. Device modules follow consistent pattern.
3. **Scope check:** Focused on HAL layer only. Calibration engine, real protocol details deferred to Phase 3+.
4. **Ambiguity check:** All requirements explicit. No dual interpretations found.

## Approved by user on 2026-04-24.
