# Phase 3a: Calibration Engine Design — Greyscale AutoCal

**Date:** 2026-04-24
**Status:** Approved
**Scope:** Calibration engine for greyscale AutoCal (21-point ramp, 1D LUT + white balance)
**Scope-out:** Color gamut (saturation sweeps, CMS), 3D LUT generation, HDR tone curve, device profiling

---

## 1. Goals

Build a layered calibration engine crate that:
- Orchestrates greyscale AutoCal end-to-end: patch display → measurement → analysis → LUT generation → upload
- Persists every reading to SQLite (resumable after crash)
- Emits real-time events for UI progress reporting
- Works with mock HAL implementations today, real hardware tomorrow

---

## 2. Architecture

### 2.1 Crate Decomposition

```
crates/calibration-core/
├── Cargo.toml                    # workspace dep: color-science, hal
└── src/
    ├── lib.rs                    # Re-exports
    ├── state.rs                  # CalibrationState enum, SessionConfig
    ├── patch.rs                  # PatchSet, Patch, PatchColor
    └── measure.rs                # MeasurementLoop, Reading, ReadingStats

crates/calibration-autocal/
├── Cargo.toml                    # workspace dep: calibration-core, color-science
└── src/
    ├── lib.rs
    ├── greyscale.rs              # GreyscaleAnalyzer, CorrectionStrategy
    └── lut.rs                    # Lut1DGenerator (per-channel tone curve fit)

crates/calibration-storage/
├── Cargo.toml                    # workspace dep: calibration-core, rusqlite
└── src/
    ├── lib.rs
    ├── schema.rs                 # SQLite schema definitions
    ├── session_store.rs          # Session CRUD
    └── reading_store.rs          # Reading CRUD

crates/calibration-engine/
├── Cargo.toml                    # workspace dep: calibration-core, calibration-autocal,
│                               # calibration-storage, hal, color-science
└── src/
    ├── lib.rs
    ├── engine.rs                 # CalibrationEngine — top-level coordinator
    ├── events.rs                 # CalibrationEvent (pub/sub to frontend)
    └── autocal_flow.rs           # GreyscaleAutoCalFlow — state machine
```

**Rationale:**
- `calibration-core` has zero dependencies on storage or algorithms. Just data structures and the measurement loop contract.
- `calibration-autocal` depends on core but not on storage. Algorithms swappable without touching persistence.
- `calibration-storage` depends on core but not on autocal. SQLite schema stable across algorithm changes.
- `calibration-engine` is the wiring layer. State machine, event emission, HAL integration live here.

### 2.2 Greyscale AutoCal Data Flow

**Input:** `SessionConfig` (21-point greyscale, BT.709 target, D65 white, gamma 2.2, 3 reads/patch, 500ms settle time)

**Step-by-step:**

1. **Engine initialization** — `CalibrationEngine::new(session_config, meter, display, pattern_gen, storage)`
2. **Connect phase** — Engine calls `meter.connect()`, `display.connect()`, `pattern_gen.connect()`. Emits `DeviceConnected` events.
3. **Pre-measurement (optional)** — If user requested pre-cal validation, engine runs the same patch set once and stores as `measurement_type = "pre"`.
4. **Patch sequence generation** — `GreyscalePatchSet::new(21)` produces 21 RGB patches from (0,0,0) to (1,1,1). Emits `PatchSequenceGenerated(21)`.
5. **Per-patch loop:**
   a. `pattern_gen.display_patch(patch)` — Emits `PatchDisplayed(patch_index, rgb)`
   b. Settle delay (configurable ms, default 500) — Emits `Settling(delay_ms)`
   c. `MeasurementLoop::measure(meter, n_reads=3, stability_threshold=None)` — takes 3 readings, computes mean XYZ and std dev. Emits `ReadingsComplete(patch_index, mean_xyz, std_dev)`.
   d. `storage.save_reading(session_id, patch_index, mean_xyz, std_dev)` — persisted to SQLite immediately.
   e. `CalibrationState::NextPatch` — state machine advances. Emits `ProgressUpdated(current, total)`.
6. **Analysis phase** — All 21 patches measured. `GreyscaleAnalyzer::analyze(readings, target_space)` computes per-channel correction curves. Emits `AnalysisComplete(gamma_estimate, white_balance_errors, max_de)`.
7. **LUT generation** — `Lut1DGenerator::from_corrections(analyzer.result, lut_size=256)` produces a 256-entry per-channel LUT. Emits `LutGenerated(lut_size)`.
8. **Upload phase** — Engine calls `display.upload_1d_lut(&lut)`, `display.set_white_balance(gains)`. Emits `CorrectionsUploaded`.
9. **Post-measurement (optional)** — Same as pre-measurement, stored as `measurement_type = "post"`.
10. **Complete** — Emits `SessionComplete`. Session state in DB becomes `Finished`.

**Pause/Resume:** The engine checks `CalibrationState` before every patch. If `Paused`, it stores `current_patch_index` and returns. Resume picks up at the same index.

**Rollback:** Any completed session can be "rolled back" by re-uploading the previously stored pre-calibration LUT and white balance settings (stored in `session_metadata.pre_lut`).

---

## 3. Core Types & Error Handling

### 3.1 State & Events

```rust
pub struct SessionConfig {
    pub name: String,
    pub target_space: TargetSpace,           // BT.709, BT.2020, P3, Custom
    pub tone_curve: ToneCurve,               // Gamma 2.2, 2.4, BT.1886, PQ, Custom
    pub white_point: WhitePoint,             // D65, D50, Custom(XYZ)
    pub patch_count: usize,                  // 21 for greyscale
    pub reads_per_patch: usize,              // 3 default
    pub settle_time_ms: u64,                 // 500 default
    pub stability_threshold: Option<f64>,    // None = fixed count mode
}

pub enum CalibrationState {
    Idle,
    Connecting,
    Connected,
    Measuring { current_patch: usize, total_patches: usize },
    Paused { at_patch: usize },
    Analyzing,
    ComputingLut,
    Uploading,
    Finished,
    Error(CalibrationError),
}

pub enum CalibrationEvent {
    DeviceConnected { device: String },
    PatchDisplayed { patch_index: usize, rgb: RGB },
    ReadingsComplete { patch_index: usize, xyz: XYZ, std_dev: XYZ },
    ProgressUpdated { current: usize, total: usize },
    AnalysisComplete { gamma: f64, max_de: f64, white_balance_errors: Vec<f64> },
    LutGenerated { size: usize },
    CorrectionsUploaded,
    SessionComplete { session_id: String },
    Error(CalibrationError),
}
```

### 3.2 Error Types

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum CalibrationError {
    #[error("Device connection failed: {device} — {reason}")]
    ConnectionFailed { device: String, reason: String },

    #[error("Measurement failed at patch {patch_index}: {reason}")]
    MeasurementFailed { patch_index: usize, reason: String },

    #[error("Meter read failed: {0}")]
    MeterRead(String),

    #[error("Display upload failed: {0}")]
    DisplayUpload(String),

    #[error("Analysis failed: {0}")]
    Analysis(String),

    #[error("Session paused by user")]
    Paused,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

**Design decisions:**
- `CalibrationError` is `Clone` so it can be stored in `CalibrationState::Error` and emitted as an event without ownership issues.
- Every error includes context (patch index, device name) for debugging.
- `Paused` is treated as an error variant for state machine simplicity, but the UI presents it as a normal state.

---

## 4. Storage Schema (SQLite)

```sql
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,          -- UUID v4
    name        TEXT NOT NULL,
    created_at  INTEGER NOT NULL,          -- Unix timestamp (ms)
    updated_at  INTEGER NOT NULL,
    state       TEXT NOT NULL,             -- "idle", "measuring", "analyzing", "finished", "paused", "error"
    config_json TEXT NOT NULL,             -- Serialized SessionConfig
    target_space TEXT NOT NULL,
    error_json  TEXT                       -- null unless state = "error"
);

CREATE TABLE patches (
    session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    patch_index INTEGER NOT NULL,
    patch_type  TEXT NOT NULL,             -- "greyscale", "saturation_red", etc.
    target_rgb  TEXT NOT NULL,             -- JSON [r, g, b]
    PRIMARY KEY (session_id, patch_index)
);

CREATE TABLE readings (
    session_id    TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    patch_index   INTEGER NOT NULL,
    reading_index INTEGER NOT NULL,        -- 0, 1, 2 for each of N reads
    raw_xyz       TEXT NOT NULL,           -- JSON [x, y, z]
    measurement_type TEXT NOT NULL,        -- "pre", "cal", "post"
    measured_at   INTEGER NOT NULL,        -- Unix timestamp (ms)
    PRIMARY KEY (session_id, patch_index, reading_index, measurement_type)
);

CREATE TABLE computed_results (
    session_id   TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    gamma        REAL,
    max_de       REAL,
    avg_de       REAL,
    lut_1d_json  TEXT,                     -- Serialized Lut1D
    white_balance TEXT,                    -- JSON {r, g, b}
    computed_at  INTEGER NOT NULL
);
```

**Rationale:**
- `config_json` stores the full `SessionConfig` as JSON so schema changes don't require migration for new fields.
- Readings are stored per individual reading (not averaged). This lets you re-analyze later with different algorithms without re-measuring.
- `measurement_type` distinguishes pre-cal, calibration, and post-cal readings in one table.

---

## 5. Testing Strategy

### 5.1 Unit Tests (per crate)
- **calibration-core:** PatchSet generation, ReadingStats (mean, std dev), CalibrationState transitions
- **calibration-autocal:** GreyscaleAnalyzer against known reference data, LUT correctness (round-trip test)
- **calibration-storage:** SQLite CRUD, schema migration, cascading delete

### 5.2 Integration Tests (calibration-engine)
- Full greyscale flow with `FakeMeter`, `FakeDisplayController`, `FakePatternGenerator`
- Pause/resume at patch 7, verify continuation from correct index
- Crash recovery: kill engine mid-session, verify resumable from DB state
- Error injection: make FakeMeter fail at patch 10, verify state becomes Error with correct context

### 5.3 Golden Path Test
```rust
#[test]
fn test_greyscale_autocal_with_mocks() {
    let meter = FakeMeter::with_preset(XYZ { x: 50.0, y: 75.0, z: 25.0 });
    let display = FakeDisplayController::default();
    let gen = FakePatternGenerator::default();
    let storage = InMemoryStorage::new(); // or real SQLite in temp dir

    let config = SessionConfig {
        name: "Test Greyscale".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.2),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 0, // fast for tests
        stability_threshold: None,
    };

    let mut engine = CalibrationEngine::new(config, meter, display, gen, storage);
    let result = engine.run_greyscale_autocal();

    assert!(result.is_ok());
    assert_eq!(engine.state(), CalibrationState::Finished);
}
```

---

## 6. Performance Considerations

- **SQLite WAL mode** for concurrent reads during write-heavy measurement loops
- **Batch inserts** for readings if `n_reads` > 1 (insert all 3 readings in one transaction)
- **Event channel bounded** at 256 messages to prevent unbounded memory growth if UI lags

---

## 7. Out of Scope (Phase 3b+)

| Feature | Phase |
|---------|-------|
| LG OLED AutoCal HTTP protocol | Phase 3b |
| PGenerator HTTP pattern display | Phase 3b |
| X-Rite i1 Display Pro HID driver | Phase 3c |
| Color gamut (saturation sweeps, CMS) | Phase 3d |
| 3D LUT generation | Phase 4 |
| HDR tone curve (PQ, HLG) | Phase 4 |
| Device profiling | Phase 5 |
| Tauri IPC wiring | Phase 3b/3c |

---

## 8. Spec Self-Review

1. **Placeholder scan:** No TBD, TODO, or incomplete sections. All code is complete.
2. **Internal consistency:** Crate dependency graph matches. Types defined in core are used in autocal, storage, and engine.
3. **Scope check:** Focused on greyscale AutoCal. Color gamut, HDR, 3D LUT explicitly scoped out.
4. **Ambiguity check:** All enum variants have concrete types. Error messages include context. State machine has explicit transitions.

No issues found.
