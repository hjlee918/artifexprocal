# ArtifexProCal v2 Architecture Design Document

**Date:** 2026-04-30
**Status:** Draft — pending review
**Scope:** Defines the plugin-style architecture for the v2 rebuild, including the three-layer design, module contracts, IPC structure, file layout, and v1 crate adaptation strategy.

**Approach:** Hybrid — Module Shell + Ported Crates (brainstormed and approved)

**First shippable increment:** `module-meter` with full `StandaloneMeterView` frontend — spot reads without a workflow engine.

---

## 1. Philosophy

The v1 architecture collapsed under its own weight because:
- `CalibrationService` became a 1000-line god object
- Frontend state was never explicitly hydrated from the backend
- Bindings were hand-maintained and drifted
- Wizard steps were built in isolation with incompatible data types
- Hardware drivers had no self-test capability

The v2 rebuild adopts a **plugin/module architecture** with strict separation of concerns:
1. **Global Settings** — persistent configuration, device inventory, user preferences
2. **Module Registry** — runtime-loaded calibration modules (meter, display, pattern generator, profiling, reporting)
3. **Workflow Engine** — state-machine-driven wizards that orchestrate modules

Each module is a first-class citizen with its own backend trait implementation, frontend UI panel, and IPC surface. No module knows about another module's internals; they communicate only through well-defined data contracts and shared state managed by the workflow engine.

**Crate naming convention:** Use hyphens consistently for v2 (`hal-meters`, `app-core`, `module-meter`, `color-science`). Rust convention.

---

## 2. Three-Layer Architecture

### 2.1 Global Settings Layer

**Responsibility:** Cross-cutting configuration that outlives any single workflow.

**Rust (`app-core` crate):**
- `SettingsStore` — SQLite-backed key-value store with typed getters/setters
- `DeviceInventory` — list of known devices (meters, displays, pattern generators) with connection history, nicknames, and capability flags
- `UserPreferences` — theme, measurement defaults, target presets

**Frontend:**
- `SettingsPanel` — React component tree for editing preferences
- `DeviceInventoryPanel` — CRUD for known devices, connection testing
- Zustand store: `useSettingsStore` — fetches full state on mount, subscribes to backend change events

**IPC Commands:**
- `settings_get(key: String) -> Option<serde_json::Value>`
- `settings_set(key: String, value: serde_json::Value) -> Result<(), SettingsError>`
- `inventory_list() -> Vec<DeviceRecord>`
- `inventory_add(device: DeviceRecord) -> Result<DeviceId, InventoryError>`
- `inventory_remove(id: DeviceId) -> Result<(), InventoryError>`
- `inventory_test_connection(id: DeviceId) -> Result<ConnectionStatus, ConnectionError>`

### 2.2 Module Registry Layer

**Responsibility:** Discover, load, and lifecycle-manage calibration modules. Each module exposes a standard trait on the backend and a standard component interface on the frontend.

#### 2.2.1 Rust Module Trait

```rust
pub trait CalibrationModule: Send + Sync {
    /// Unique module identifier (e.g., "meter", "display", "pattern_gen", "profiling", "reporting")
    fn module_id(&self) -> &'static str;

    /// Human-readable display name
    fn display_name(&self) -> &'static str;

    /// Module capabilities — used by the workflow engine to determine which modules can participate in a given workflow
    fn capabilities(&self) -> Vec<ModuleCapability>;

    /// Lifecycle: called once when the module is registered at app startup
    fn initialize(&mut self, ctx: &ModuleContext) -> Result<(), ModuleError>;

    /// Lifecycle: called when the module is about to participate in an active workflow
    fn activate(&mut self, workflow_id: WorkflowId) -> Result<(), ModuleError>;

    /// Lifecycle: called when the workflow ends or the module is swapped out
    fn deactivate(&mut self) -> Result<(), ModuleError>;

    /// Return the set of IPC commands this module exposes (see §3)
    fn commands(&self) -> &'static [ModuleCommandDef];

    /// Handle a command invocation from the frontend
    fn handle_command(
        &mut self,
        cmd: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, CommandError>;

    /// Subscribe to backend events (measurement complete, display state change, etc.)
    fn event_stream(&self) -> Option<tokio::sync::broadcast::Receiver<ModuleEvent>>;
}
```

#### 2.2.2 ModuleContext

The `ModuleContext` provides access to shared services without direct coupling:

```rust
pub struct ModuleContext {
    pub settings: Arc<SettingsStore>,
    pub inventory: Arc<DeviceInventory>,
    pub storage: Arc<dyn CalibrationStorage>,
    pub event_bus: Arc<EventBus>,
    pub logger: Arc<dyn Logger>,
}
```

No module holds a concrete reference to another module. If the MeterModule needs to log a reading, it uses `ctx.event_bus.publish()`. If the ReportingModule needs readings, it queries `ctx.storage`.

#### 2.2.3 React Module Interface

Every frontend module exposes a standard interface object registered at app startup:

```typescript
interface FrontendModule {
  id: string;
  displayName: string;
  capabilities: ModuleCapability[];

  /** Root settings/config panel for this module (shown in global settings) */
  SettingsPanel: React.ComponentType<ModuleSettingsProps>;

  /** Live monitoring panel (shown during active workflow) */
  MonitorPanel: React.ComponentType<ModuleMonitorProps>;

  /** Quick-action button strip for the dashboard */
  QuickActions: React.ComponentType<ModuleQuickActionProps>;
}
```

The `ModuleRegistry` (a Zustand store) maintains the list of registered modules. The workflow engine queries this registry to build UI dynamically.

### 2.3 Workflow Engine Layer

**Responsibility:** State-machine-driven orchestration of calibration workflows. The engine knows which modules are needed for a workflow, advances the state machine, and coordinates inter-module data flow.

#### 2.3.1 Workflow State Machine

Every workflow is a finite state machine with a single canonical state type:

```rust
pub struct WorkflowState {
    pub workflow_id: WorkflowId,
    pub workflow_type: WorkflowType,     // AutoCal, Manual, Profiling, Verification
    pub current_step: WorkflowStep,
    pub step_history: Vec<WorkflowStep>,
    pub module_states: HashMap<String, serde_json::Value>, // per-module opaque state
    pub shared_data: SharedWorkflowData,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum WorkflowType {
    AutoCal,
    Manual,
    Profiling,
    Verification,
    StandaloneMeter,  // meter-only mode for spot reads
}

pub enum WorkflowStep {
    // Common steps
    Setup,              // Select devices, targets, picture mode
    Connection,       // Connect meter, display, pattern generator
    PreMeasurement,     // Pre-calibration measurement (grayscale + primaries)
    Processing,         // Generate LUTs / correction matrices
    PostMeasurement,    // Post-calibration verification
    Review,             // Analysis, reports, decision to iterate
    Complete,           // Save session, disconnect devices
    Error,              // Terminal error state with recovery options
}
```

#### 2.3.2 Step Contract

Every step receives the same `WorkflowState` and emits the same `WorkflowAction`:

```rust
pub enum WorkflowAction {
    Next,                    // Advance to next step
    Back,                    // Return to previous step
    SkipTo(WorkflowStep),    // Jump to a specific step (e.g., skip pre-measurement)
    Abort,                   // Terminate workflow, clean up
    SetModuleState { module_id: String, state: serde_json::Value },
    AppendSharedData { key: String, value: serde_json::Value },
    Error { message: String, recoverable: bool },
}
```

**Rule:** No step defines its own state type. All state lives in `WorkflowState.module_states` or `WorkflowState.shared_data`. This prevents the data-contract mismatch that broke v1.

#### 2.3.3 Frontend Wizard Shell

The React `WizardShell` component:
1. Subscribes to backend `WorkflowState` on mount (hydration rule #1)
2. Renders the current step's UI based on `current_step`
3. Sends `WorkflowAction` to the backend via IPC
4. Displays a progress sidebar showing all steps with completion status

```typescript
interface WizardShellProps {
  workflowType: WorkflowType;
  onComplete: (sessionId: string) => void;
  onAbort: () => void;
}
```

---

## 3. IPC Command Structure

### 3.1 Thin Tauri Command Layer

The Tauri command handlers (`src-tauri/src/ipc/commands.rs`) are thin delegators — under 100 lines each. They:
1. Parse incoming request
2. Route to the appropriate module or workflow engine
3. Return result or error

```rust
#[tauri::command]
#[specta::specta]  // auto-generates TypeScript binding
async fn module_command(
    state: tauri::State<'_, AppState>,
    module_id: String,
    command: String,
    payload: serde_json::Value,
) -> Result<serde_json::Value, CommandError> {
    let registry = state.module_registry.lock().await;
    let module = registry.get(&module_id)
        .ok_or(CommandError::ModuleNotFound(module_id))?;
    module.handle_command(&command, payload)
}
```

**Rule:** No business logic in command handlers. No `CalibrationService` singleton.

### 3.2 Auto-Generated Bindings

`tauri-specta` generates TypeScript bindings on every `cargo build`:
- Commands: `invoke("module_command", { moduleId, command, payload })` with full type safety
- Events: `listen("module_event", (event) => { ... })` with typed payload

The build pipeline fails if bindings are out of sync. Manual `bindings.ts` is forbidden.

### 3.3 Event Streams

Backend-to-frontend events use `tokio::sync::broadcast` channels:

```rust
pub enum AppEvent {
    WorkflowStateChanged(WorkflowState),
    ModuleEvent { module_id: String, event: ModuleEvent },
    MeasurementProgress(MeasurementProgress),
    DeviceConnectionChanged { device_id: DeviceId, status: ConnectionStatus },
    Error { source: String, message: String },
}
```

Frontend subscribes via Tauri's `listen()` API. No `blocking_recv` in UI-facing code (rule #6).

---

## 4. Data Contracts

### 4.1 MeasurementResult Location

`MeasurementResult` lives in the **`color-science` crate**, not `app-core`. It is a color science concept and many modules depend on it. `app-core` re-exports it for convenience.

```rust
// crates/color-science/src/measurement.rs
use specta::Type;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MeasurementResult {
    // ... full spec in meter-module.md §2.3 ...
}
```

### 4.2 Color Science Types

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Xyz { pub x: f64, pub y: f64, pub z: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct XyY { pub x: f64, pub y: f64, pub y_lum: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Lab { pub l: f64, pub a: f64, pub b: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct LCh { pub l: f64, pub c: f64, pub h: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct ICtCp { pub i: f64, pub ct: f64, pub cp: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Rgb<T> { pub r: T, pub g: T, pub b: T }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub enum RgbSpace { Srgb, Rec709, Rec2020, DciP3 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub enum WhitePoint { D50, D55, D65, D75, Custom(XyY) }
```

---

## 5. V1 Crate Adaptation Strategy

### 5.1 Why Not Rewrite

The v1 archive contains 9 working crates with real, tested code. The individual crates are not the problem — the integration layer (`CalibrationService`, manual bindings, frontend state mismatch) is. Rewriting everything would waste months of hardware driver work and violate Lesson #8 (every phase ships end-to-end).

### 5.2 Adaptation Pattern: Wrap, Don't Rewrite

Each v1 crate becomes a v2 module by adding a **wrapper crate** that implements the `CalibrationModule` trait and adapts the v1 API:

```
v1 crate              →   v2 wrapper crate        →   Module trait impl
─────────────────────────────────────────────────────────────────────
hal_meters            →   module-meter            →   CalibrationModule for "meter"
hal-displays          →   module-display          →   CalibrationModule for "display"
hal-patterns          →   module-pattern          →   CalibrationModule for "pattern_gen"
calibration-core      →   calibration-engine        →   WorkflowEngine (not a module)
calibration-storage   →   calibration-storage     →   shared service (no module trait)
color-science         →   color-science           →   shared types (no module trait)
reporting             →   module-reporting        →   CalibrationModule for "reporting"
```

**Example: `hal_meters` → `module-meter`**

The v1 `hal_meters` crate has:
- `I1DisplayPro` struct with native HID methods (blocking I/O)
- `ArgyllMeter` struct with PTY subprocess methods (blocking I/O)
- `i1d3_unlock` module with challenge-response protocol
- `argyll_adapter` module with PTY wrapper

The v2 `module-meter` crate:
1. Imports `hal_meters` as a dependency
2. Wraps `I1DisplayPro` and `ArgyllMeter` in an async adapter
3. Exposes a `CalibrationModule` implementation
4. Translates between v1 error types and v2 `CommandError`

```rust
// crates/module-meter/src/driver_adapter.rs
use hal_meters::{I1DisplayPro, ArgyllMeter, Meter as V1Meter};
use color_science::{MeasurementResult, Xyz};
use tokio::task;

pub struct AsyncI1DisplayPro {
    inner: I1DisplayPro,
}

impl AsyncI1DisplayPro {
    pub async fn read_xyz(&mut self) -> Result<Xyz, MeterError> {
        // v1 I1DisplayPro::read_xyz() is blocking (HID I/O)
        // Wrap in spawn_blocking to make it async
        let mut inner = self.inner; // or Arc<Mutex<_>>
        task::spawn_blocking(move || inner.read_xyz())
            .await
            .map_err(|e| MeterError::JoinError(e.to_string()))?
    }
}
```

### 5.3 Adding `specta::Type` Derives to V1 Types

V1 types (`Xyz`, `Lab`, `MeasurementResult`) do not derive `specta::Type`, so `tauri-specta` cannot auto-generate TypeScript bindings for them.

**Strategy:** Add `specta::Type` to `color-science` crate types. `specta` works with `serde` — if a type already derives `Serialize + Deserialize`, adding `specta::Type` is usually a one-line change.

**Known gotchas:**
- `Vec<f64>` (spectrum data): `specta` handles this natively (`Vec<f64>` → `number[]`)
- `DateTime<Utc>`: use `specta::Type` with `specta::datatype::NamedType` or a newtype wrapper
- `HashMap<String, serde_json::Value>`: `specta` supports this as `Record<string, any>`
- Generic types like `Rgb<T>`: `specta` supports generic type parameters

If a type cannot derive `specta::Type` (rare), use a **DTO (Data Transfer Object)** in the IPC layer:

```rust
#[derive(Serialize, Deserialize, specta::Type)]
pub struct MeasurementResultDto {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    // ... flattened fields
}

impl From<MeasurementResult> for MeasurementResultDto { ... }
```

**Rule:** Prefer adding `specta::Type` to the canonical type. Only use DTOs as a last resort.

### 5.4 Async Adapter for Blocking V1 Drivers

V1 drivers use blocking I/O (HID USB reads, PTY reads). v2 requires async because:
- Tauri commands are async (`async fn module_command`)
- The event loop uses `tokio::sync::broadcast`
- UI must not freeze during a 5-second meter read

**Pattern:** Every blocking v1 driver gets an async wrapper using `tokio::task::spawn_blocking`:

```rust
pub struct AsyncMeterAdapter<M: V1Meter + Send + 'static> {
    inner: Arc<Mutex<M>>,
}

impl<M: V1Meter + Send + 'static> AsyncMeterAdapter<M> {
    pub async fn read_xyz(&mut self) -> Result<Xyz, MeterError> {
        let inner = self.inner.clone();
        task::spawn_blocking(move || {
            let mut guard = inner.lock().unwrap();
            guard.read_xyz()
        })
        .await
        .map_err(|e| MeterError::JoinError(e.to_string()))?
    }

    pub async fn probe(&mut self) -> Result<bool, MeterError> {
        let inner = self.inner.clone();
        task::spawn_blocking(move || {
            let mut guard = inner.lock().unwrap();
            guard.probe()
        })
        .await
        .map_err(|e| MeterError::JoinError(e.to_string()))?
    }
}
```

**For ArgyllCMS PTY:** The PTY wrapper in v1 used `std::process::Command` and blocking `BufReader::read_line`. The v2 adapter:
1. Spawns `spotread` via `tokio::process::Command`
2. Uses `tokio::io::AsyncBufReadExt` for async line reading
3. Times out with `tokio::time::timeout`

```rust
pub struct AsyncArgyllMeter {
    child: tokio::process::Child,
    reader: tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
}

impl AsyncArgyllMeter {
    pub async fn read_xyz(&mut self) -> Result<Xyz, MeterError> {
        let line = tokio::time::timeout(
            Duration::from_secs(30),
            self.reader.next_line()
        ).await??;
        parse_spotread_output(&line.ok_or(MeterError::NoOutput)?)
    }
}
```

### 5.5 Porting Order

| Phase | V1 Crate | V2 Module | Effort |
|-------|----------|-----------|--------|
| 1 | `color-science` | `color-science` + `specta::Type` | Low |
| 2 | `hal` | `hal` (trait crate, minimal changes) | Low |
| 3 | `hal_meters` | `module-meter` | Medium (async adapters) |
| 4 | `hal-displays` | `module-display` | Medium |
| 5 | `hal-patterns` | `module-pattern` | Low |
| 6 | `calibration-core` + `calibration-engine` | `calibration-engine` (workflow engine) | High |
| 7 | `reporting` | `module-reporting` | Medium |
| 8 | `calibration-storage` | `calibration-storage` (shared service) | Low |

---

## 6. Risk Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `specta::Type` derive fails on complex types | Medium | High | Add custom type registration for `Vec<f64>`; `specta` supports this. Use DTOs as fallback. |
| `tokio::sync::broadcast` drops events if frontend is slow | High | Medium | Use bounded channels with `lag` tolerance for progress events; use `tokio::sync::mpsc` for ordered events. Frontend subscribes with `listen()` which is callback-based, not blocking. |
| Three.js already in `package.json` tempts early 3D viz | Medium | Medium | Remove `three`, `@react-three/fiber`, `@react-three/drei` from `package.json` now. Re-add in Phase 5 when real LUT data exists. |
| ArgyllCMS subprocess PTY code from v1 doesn't port cleanly to async | Medium | High | v1 PTY code used blocking I/O; v2 uses `tokio::process::Command` + `tokio::io::AsyncBufReadExt`. Test PTY adapter with mock `spotread` script. |
| i1 Display Pro HID `read_report_with_timeout` is blocking | High | High | Wrap in `tokio::task::spawn_blocking`; emit events via channel. The HID read itself remains sync, but the surrounding module is async. |

---

## 7. File / Folder Structure

```
artifexprocal/
├── Cargo.toml                    # workspace root
├── package.json
├── vite.config.ts
├── tauri.conf.json
├── src/                          # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── styles.css
│   ├── stores/
│   │   ├── useSettingsStore.ts
│   │   ├── useModuleRegistryStore.ts
│   │   └── useWorkflowStore.ts   # hydrates from backend on mount
│   ├── components/
│   │   ├── common/               # shared UI primitives
│   │   ├── settings/
│   │   ├── wizard/
│   │   │   ├── WizardShell.tsx
│   │   │   ├── StepSidebar.tsx
│   │   │   └── steps/
│   │   │       ├── SetupStep.tsx
│   │   │       ├── ConnectionStep.tsx
│   │   │       ├── MeasurementStep.tsx
│   │   │       ├── ProcessingStep.tsx
│   │   │       ├── ReviewStep.tsx
│   │   │       └── CompleteStep.tsx
│   │   ├── dashboard/
│   │   └── modules/
│   │       └── meter/
│   │           ├── MeterSettingsPanel.tsx
│   │           ├── MeterMonitorPanel.tsx
│   │           ├── MeterQuickActions.tsx
│   │           ├── StandaloneMeterView.tsx
│   │           └── MeasurementTable.tsx
│   ├── modules/
│   │   └── meter/                # frontend module logic
│   │       ├── index.ts          # exports FrontendModule object
│   │       ├── api.ts            # typed wrappers for meter IPC commands
│   │       └── types.ts          # module-specific TypeScript types
│   └── bindings/                 # auto-generated by tauri-specta (gitignored)
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── ipc/
│   │   │   ├── commands.rs       # thin Tauri command handlers (<100 lines each)
│   │   │   └── events.rs         # event emission helpers
│   │   ├── app_state.rs          # AppState struct (module registry, settings, event bus)
│   │   └── modules/
│   │       └── mod.rs            # CalibrationModule trait + registry
│   └── capabilities/
├── crates/                       # Rust workspace crates
│   ├── app-core/                 # shared types, settings, errors, CalibrationModule trait
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── settings.rs
│   │       ├── inventory.rs
│   │       ├── errors.rs
│   │       ├── module.rs         # CalibrationModule trait, ModuleContext, ModuleCapability
│   │       └── event_bus.rs      # EventBus, AppEvent
│   ├── color-science/            # XYZ/Lab/LCh/ICtCp conversions, DeltaE, gamut math + MeasurementResult
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs          # Xyz, XyY, Lab, LCh, ICtCp, Rgb, WhitePoint, RgbSpace + specta::Type
│   │       ├── conversion.rs
│   │       ├── delta_e.rs
│   │       ├── gamut.rs
│   │       ├── tone_curves.rs
│   │       └── measurement.rs    # MeasurementResult (re-exported by app-core)
│   ├── hal/                      # hardware abstraction traits
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── meter.rs          # Meter trait (v1 trait, preserved)
│   │       ├── display.rs        # DisplayController trait
│   │       ├── pattern_gen.rs    # PatternGenerator trait
│   │       └── types.rs          # Lut1D, Lut3D, PictureMode, etc.
│   ├── hal-meters/               # meter driver implementations (v1 code, archived logic)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── i1_display_pro.rs
│   │       ├── i1_pro_2.rs
│   │       ├── argyll_adapter.rs
│   │       ├── i1d3_unlock.rs
│   │       ├── usb_hid.rs
│   │       └── mock.rs
│   ├── hal-displays/             # display driver implementations
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lg_oled.rs
│   │       ├── sony_projector.rs
│   │       └── mock.rs
│   ├── hal-patterns/             # pattern generator implementations
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pgenerator.rs
│   │       ├── lg_internal.rs
│   │       └── mock.rs
│   ├── calibration-engine/       # workflow engine + patch sequencer
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── workflow.rs       # WorkflowState machine
│   │       ├── sequencer.rs      # patch generation (grayscale, saturation, etc.)
│   │       ├── autocal.rs        # AutoCal logic
│   │       └── events.rs         # EventBus, MeasurementProgress
│   ├── calibration-storage/      # SQLite persistence
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs
│   │       ├── sessions.rs
│   │       ├── readings.rs
│   │       └── profiling.rs
│   ├── module-meter/             # MeterModule implementation (v2 wrapper around hal-meters)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── meter_module.rs   # CalibrationModule impl for meters
│   │       ├── detection.rs      # USB enumeration, instrument discovery
│   │       ├── commands.rs       # command handlers exposed via IPC
│   │       ├── state.rs          # per-module runtime state
│   │       └── driver_adapter.rs # async adapters for v1 hal-meters drivers
│   ├── module-display/           # DisplayModule (Phase 3+)
│   ├── module-pattern/           # PatternModule (Phase 3+)
│   ├── module-profiling/         # ProfilingModule (Phase 5+)
│   ├── module-reporting/         # ReportingModule (Phase 4+)
│   └── module-manual/            # Manual measurement mode (Phase 5+)
├── docs/
│   ├── architecture/
│   │   └── v2-architecture.md   # this document
│   ├── modules/
│   │   └── meter-module.md      # MeterModule design spec
│   ├── LESSONS_LEARNED.md
│   └── LG_LUT_FORMAT.md
└── archive/v1/                   # v1 codebase (frozen)
```

---

## 8. Key Architectural Rules (from v1 Lessons)

| # | Rule | Enforcement |
|---|------|-------------|
| 1 | Store hydrates on mount | Every Zustand store calls `invoke("settings_get_all")` or equivalent in its initializer |
| 2 | IPC handlers are thin delegators | `commands.rs` under 100 lines per handler; `clippy` lint for function length |
| 3 | Bindings auto-generated | `tauri-specta` in build script; CI fails if `bindings/` dirty |
| 4 | State machine before UI | `WorkflowState` and `WorkflowAction` defined before any React step component |
| 5 | No Three.js until real data flows | Remove `three` from `package.json` now; SVG/Canvas for CIE diagrams in Phases 1–4 |
| 6 | No `blocking_recv` in UI-facing code | `tokio::time::timeout` on all event loops; frontend uses `listen()` callbacks |
| 7 | Pattern generator is a first-class device | Its own module, inventory entry, connection UI, and `CalibrationModule` impl |
| 8 | Every phase ships end-to-end | Each phase must produce a working, testable increment with `npm run dev` |
| 9 | Correction matrix is HAL concern | Applied at `Meter` trait level, not duplicated across calibration flows |
| 10 | Hardware drivers need self-test | Every driver has a `probe()` method for connectivity verification |
| 11 | `.md` is truth; `.docx` is generated | `scripts/md_to_docx.py` runs at commit time; never hand-edit `.docx` |
| 12 | Fewer phases, larger working increments | Phase 1: Module shell + MeterModule standalone; Phase 2: Display + Pattern modules; Phase 3: Workflow engine + AutoCal; Phase 4: History + reporting; Phase 5: 3D LUT + manual + profiling |

---

## 9. Approval Checklist

Before implementation begins, confirm:
- [ ] Three-layer architecture (Global Settings → Module Registry → Workflow Engine) is acceptable
- [ ] `CalibrationModule` trait covers all planned modules
- [ ] Crate naming convention (hyphens) is acceptable
- [ ] `MeasurementResult` location in `color-science` crate is acceptable
- [ ] V1 adaptation strategy (wrap, don't rewrite) is acceptable
- [ ] Async adapter pattern (`spawn_blocking` for HID, `tokio::process` for PTY) is acceptable
- [ ] File/folder structure is acceptable
- [ ] Phase plan (12 rules, §8) is acceptable
- [ ] Risk mitigations (§6) are acceptable
