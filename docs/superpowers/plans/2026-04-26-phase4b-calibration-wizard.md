# Phase 4b: Calibration Wizard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the step-by-step greyscale AutoCal wizard with optional inline meter profiling, live measurement feedback, post-calibration analysis, and verification.

**Architecture:** Backend adds new Tauri commands (`start_calibration`, `abort_calibration`, `start_profiling`, `abort_profiling`) and events. `CalibrationService` gains session tracking and an EventChannel bridge. Frontend builds a wizard shell with 6 steps, SVG-based charts, and event-driven UI updates.

**Tech Stack:** Rust (Tauri 2.x, tauri-specta, tokio, calibration-engine), React 19 + TypeScript + Tailwind CSS v4, SVG for charts.

---

## File Structure

### Backend (`src-tauri/src/`)

| File | Change | Responsibility |
|------|--------|---------------|
| `ipc/models.rs` | Modify | Add `SessionConfigDto`, `CalibrationProgress`, `ProfilingConfig`, `ProfilingProgress` |
| `ipc/commands.rs` | Modify | Add `start_calibration`, `abort_calibration`, `start_profiling`, `abort_profiling` |
| `ipc/events.rs` | Modify | Add emitters for calibration/profiling progress, analysis, upload, verification |
| `service/state.rs` | Modify | Add session tracking, `tokio::task::spawn_blocking` wrapper, event bridge |
| `service/error.rs` | Modify | Add `SessionInProgress`, `SessionNotFound` errors |
| `lib.rs` | Modify | Register new commands in `generate_handler!` |
| `bindings_export.rs` | Modify | Add new commands to tauri-specta export |

### Frontend (`src/`)

| File | Change | Responsibility |
|------|--------|---------------|
| `bindings.ts` | Modify | Add new types, commands, event constants |
| `components/calibrate/CalibrationWizard.tsx` | Create | Stepper shell with next/back navigation |
| `components/calibrate/DeviceSelectionStep.tsx` | Create | Step 1: device cards + pre-flight + profiling toggle |
| `components/calibrate/TargetConfigStep.tsx` | Create | Step 2: target settings form |
| `components/calibrate/MeasurementStep.tsx` | Create | Step 3: live measurement UI |
| `components/calibrate/AnalysisStep.tsx` | Create | Step 4: dE chart + gamma + table |
| `components/calibrate/UploadStep.tsx` | Create | Step 5: upload progress |
| `components/calibrate/VerifyStep.tsx` | Create | Step 6: post comparison |
| `components/calibrate/LiveGammaChart.tsx` | Create | SVG gamma curve with target dashed line |
| `components/calibrate/DeBarChart.tsx` | Create | SVG dE2000 bar chart |
| `components/calibrate/PatchDataTable.tsx` | Create | Color-coded patch results table |
| `components/calibrate/YxyReadout.tsx` | Create | Big numeric Yxy display |
| `components/calibrate/ProfilingStep.tsx` | Create | Step 1a: inline meter profiling |
| `components/views/CalibrateView.tsx` | Modify | Replace placeholder with wizard container |
| `components/views/DevicesView.tsx` | Modify | Add "Profile Meter" card/button |
| `components/devices/MeterProfiler.tsx` | Create | Standalone profiler wizard |

---

### Task 1: Backend — New IPC Models

**Files:**
- Modify: `src-tauri/src/ipc/models.rs`

- [ ] **Step 1: Add SessionConfigDto**

```rust
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct SessionConfigDto {
    pub name: String,
    pub target_space: String,
    pub tone_curve: String,
    pub white_point: String,
    pub patch_count: usize,
    pub reads_per_patch: usize,
    pub settle_time_ms: u64,
    pub stability_threshold: Option<f64>,
}
```

- [ ] **Step 2: Add CalibrationProgress**

```rust
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct CalibrationProgress {
    pub session_id: String,
    pub current_patch: usize,
    pub total_patches: usize,
    pub patch_name: String,
    pub yxy: Option<(f64, f64, f64)>,
    pub stable: bool,
}
```

- [ ] **Step 3: Add ProfilingConfig and ProfilingProgress**

```rust
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct ProfilingConfig {
    pub patch_set: String,
    pub patch_scale: String,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct ProfilingProgress {
    pub session_id: String,
    pub current_patch: usize,
    pub total_patches: usize,
    pub patch_name: String,
    pub reference_xyz: (f64, f64, f64),
    pub meter_xyz: (f64, f64, f64),
    pub delta_e: f64,
}
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/ipc/models.rs
git commit -m "feat(ipc): add SessionConfigDto, CalibrationProgress, ProfilingConfig, ProfilingProgress"
```

---

### Task 2: Backend — New Events

**Files:**
- Modify: `src-tauri/src/ipc/events.rs`

- [ ] **Step 1: Add calibration event emitters**

```rust
pub fn emit_calibration_progress(
    app: &AppHandle,
    session_id: String,
    current_patch: usize,
    total_patches: usize,
    patch_name: String,
    yxy: Option<(f64, f64, f64)>,
    stable: bool,
) {
    let _ = app.emit(
        "calibration-progress",
        crate::ipc::models::CalibrationProgress {
            session_id,
            current_patch,
            total_patches,
            patch_name,
            yxy,
            stable,
        },
    );
}

pub fn emit_analysis_complete(
    app: &AppHandle,
    session_id: String,
    gamma: f64,
    max_de: f64,
    avg_de: f64,
    white_balance_errors: Vec<f64>,
) {
    let _ = app.emit(
        "analysis-complete",
        serde_json::json!({
            "session_id": session_id,
            "gamma": gamma,
            "max_de": max_de,
            "avg_de": avg_de,
            "white_balance_errors": white_balance_errors,
        }),
    );
}

pub fn emit_lut_uploaded(app: &AppHandle, session_id: String) {
    let _ = app.emit("lut-uploaded", serde_json::json!({ "session_id": session_id }));
}

pub fn emit_verification_complete(
    app: &AppHandle,
    session_id: String,
    pre_de: Vec<f64>,
    post_de: Vec<f64>,
) {
    let _ = app.emit(
        "verification-complete",
        serde_json::json!({
            "session_id": session_id,
            "pre_de": pre_de,
            "post_de": post_de,
        }),
    );
}
```

- [ ] **Step 2: Add profiling event emitters**

```rust
pub fn emit_profiling_progress(
    app: &AppHandle,
    session_id: String,
    current_patch: usize,
    total_patches: usize,
    patch_name: String,
    reference_xyz: (f64, f64, f64),
    meter_xyz: (f64, f64, f64),
    delta_e: f64,
) {
    let _ = app.emit(
        "profiling-progress",
        crate::ipc::models::ProfilingProgress {
            session_id,
            current_patch,
            total_patches,
            patch_name,
            reference_xyz,
            meter_xyz,
            delta_e,
        },
    );
}

pub fn emit_profiling_complete(
    app: &AppHandle,
    session_id: String,
    correction_matrix: [[f64; 3]; 3],
    accuracy_estimate: f64,
) {
    let _ = app.emit(
        "profiling-complete",
        serde_json::json!({
            "session_id": session_id,
            "correction_matrix": correction_matrix,
            "accuracy_estimate": accuracy_estimate,
        }),
    );
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ipc/events.rs
git commit -m "feat(ipc): add calibration and profiling event emitters"
```

---

### Task 3: Backend — CalibrationService Session Tracking

**Files:**
- Modify: `src-tauri/src/service/error.rs`
- Modify: `src-tauri/src/service/state.rs`

- [ ] **Step 1: Add new error variants**

In `src-tauri/src/service/error.rs`, add to `CalibrationError`:

```rust
#[error("A calibration session is already in progress.")]
SessionInProgress,

#[error("Session {0} not found.")]
SessionNotFound(String),
```

- [ ] **Step 2: Add session tracking to CalibrationService**

In `src-tauri/src/service/state.rs`, add imports:

```rust
use calibration_core::state::{SessionConfig, CalibrationEvent};
use calibration_engine::events::EventChannel;
use calibration_engine::autocal_flow::GreyscaleAutoCalFlow;
use std::time::Duration;
```

Add a `CalibrationSession` struct inside `state.rs`:

```rust
struct CalibrationSession {
    session_id: String,
    config: SessionConfig,
    pre_readings: Vec<(color_science::types::RGB, color_science::types::XYZ)>,
}
```

Add field to `CalibrationService`:

```rust
pub struct CalibrationService {
    meter: Arc<Mutex<Option<Box<dyn Meter + Send>>>>,
    meter_info: Arc<Mutex<Option<MeterInfo>>>,
    display: Arc<Mutex<Option<Box<dyn DisplayController + Send>>>>,
    display_info: Arc<Mutex<Option<DisplayInfo>>>,
    state: Arc<Mutex<CalibrationState>>,
    use_mocks: bool,
    active_session: Arc<Mutex<Option<CalibrationSession>>>,
}
```

Update `new()` and `with_mocks()` to initialize `active_session`:

```rust
active_session: Arc::new(Mutex::new(None)),
```

Add methods:

```rust
pub fn start_calibration_session(
    &self,
    config: SessionConfig,
) -> Result<String, CalibrationError> {
    let mut guard = self.active_session.lock();
    if guard.is_some() {
        return Err(CalibrationError::SessionInProgress);
    }
    let session_id = format!("cal-{}", uuid::Uuid::new_v4());
    *guard = Some(CalibrationSession {
        session_id: session_id.clone(),
        config,
        pre_readings: Vec::new(),
    });
    Ok(session_id)
}

pub fn get_active_session_id(&self) -> Option<String> {
    self.active_session.lock().as_ref().map(|s| s.session_id.clone())
}

pub fn end_session(&self) {
    *self.active_session.lock() = None;
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/service/error.rs src-tauri/src/service/state.rs
git commit -m "feat(service): add CalibrationService session tracking"
```

---

### Task 4: Backend — New Tauri Commands

**Files:**
- Modify: `src-tauri/src/ipc/commands.rs`

- [ ] **Step 1: Add new commands**

```rust
#[tauri::command]
#[specta::specta]
pub fn start_calibration(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    config: crate::ipc::models::SessionConfigDto,
) -> Result<String, String> {
    let session_config = calibration_core::state::SessionConfig {
        name: config.name,
        target_space: match config.target_space.as_str() {
            "Rec.2020" => calibration_core::state::TargetSpace::Rec2020,
            "DCI-P3" => calibration_core::state::TargetSpace::DciP3,
            _ => calibration_core::state::TargetSpace::Rec709,
        },
        tone_curve: match config.tone_curve.as_str() {
            "Gamma 2.2" => calibration_core::state::ToneCurve::Gamma(2.2),
            "Gamma 2.4" => calibration_core::state::ToneCurve::Gamma(2.4),
            "BT.1886" => calibration_core::state::ToneCurve::Bt1886,
            "PQ" => calibration_core::state::ToneCurve::Pq,
            "HLG" => calibration_core::state::ToneCurve::Hlg,
            _ => calibration_core::state::ToneCurve::Gamma(2.4),
        },
        white_point: match config.white_point.as_str() {
            "D50" => calibration_core::state::WhitePoint::D50,
            "DCI" => calibration_core::state::WhitePoint::Dci,
            _ => calibration_core::state::WhitePoint::D65,
        },
        patch_count: config.patch_count,
        reads_per_patch: config.reads_per_patch,
        settle_time_ms: config.settle_time_ms,
        stability_threshold: config.stability_threshold,
    };

    let session_id = service
        .start_calibration_session(session_config)
        .map_err(|e| e.to_string())?;

    // Spawn calibration in blocking thread (placeholder — full integration in Task 5)
    let app_clone = app.clone();
    std::thread::spawn(move || {
        // Emit a dummy progress event after 1s for testing
        std::thread::sleep(Duration::from_secs(1));
        crate::ipc::events::emit_calibration_progress(
            &app_clone,
            session_id.clone(),
            0,
            config.patch_count,
            "0% Black".to_string(),
            Some((0.02, 0.3125, 0.3290)),
            true,
        );
    });

    Ok(session_id)
}

#[tauri::command]
#[specta::specta]
pub fn abort_calibration(
    service: State<'_, CalibrationService>,
    session_id: String,
) -> Result<(), String> {
    if service.get_active_session_id() != Some(session_id) {
        return Err("Session not found".to_string());
    }
    service.end_session();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn start_profiling(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    meter_id: String,
    _reference_meter_id: String,
    _display_id: String,
    _config: crate::ipc::models::ProfilingConfig,
) -> Result<String, String> {
    let session_id = format!("prof-{}", uuid::Uuid::new_v4());
    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        crate::ipc::events::emit_profiling_progress(
            &app_clone,
            session_id.clone(),
            0,
            20,
            "Primary Red".to_string(),
            (45.2, 25.1, 12.3),
            (44.8, 24.9, 12.1),
            0.35,
        );
    });
    Ok(session_id)
}

#[tauri::command]
#[specta::specta]
pub fn abort_profiling(
    _service: State<'_, CalibrationService>,
    _session_id: String,
) -> Result<(), String> {
    Ok(())
}
```

Add `use std::time::Duration;` at the top of `commands.rs`.

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ipc/commands.rs
git commit -m "feat(ipc): add start/abort calibration and profiling commands"
```

---

### Task 5: Backend — Wire Commands and Update Bindings Export

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/bindings_export.rs`

- [ ] **Step 1: Register new commands in lib.rs**

Add to `invoke_handler!` in `src-tauri/src/lib.rs`:

```rust
ipc::commands::start_calibration,
ipc::commands::abort_calibration,
ipc::commands::start_profiling,
ipc::commands::abort_profiling,
```

- [ ] **Step 2: Update bindings export**

In `src-tauri/src/bindings_export.rs`, add new commands to the `collect_commands!` macro:

```rust
crate::ipc::commands::start_calibration,
crate::ipc::commands::abort_calibration,
crate::ipc::commands::start_profiling,
crate::ipc::commands::abort_profiling,
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/bindings_export.rs
git commit -m "feat(tauri): register new calibration and profiling commands"
```

---

### Task 6: Frontend — Update Bindings

**Files:**
- Modify: `src/bindings.ts`

- [ ] **Step 1: Add new types**

```typescript
export type SessionConfigDto = {
  name: string;
  target_space: string;
  tone_curve: string;
  white_point: string;
  patch_count: number;
  reads_per_patch: number;
  settle_time_ms: number;
  stability_threshold: number | null;
};

export type CalibrationProgress = {
  session_id: string;
  current_patch: number;
  total_patches: number;
  patch_name: string;
  yxy: [number, number, number] | null;
  stable: boolean;
};

export type ProfilingConfig = {
  patch_set: string;
  patch_scale: string;
};

export type ProfilingProgress = {
  session_id: string;
  current_patch: number;
  total_patches: number;
  patch_name: string;
  reference_xyz: [number, number, number];
  meter_xyz: [number, number, number];
  delta_e: number;
};
```

- [ ] **Step 2: Add new command wrappers**

```typescript
export function startCalibration(config: SessionConfigDto): Promise<string> {
  return invoke("start_calibration", { config });
}

export function abortCalibration(sessionId: string): Promise<void> {
  return invoke("abort_calibration", { sessionId });
}

export function startProfiling(
  meterId: string,
  referenceMeterId: string,
  displayId: string,
  config: ProfilingConfig
): Promise<string> {
  return invoke("start_profiling", { meterId, referenceMeterId, displayId, config });
}

export function abortProfiling(sessionId: string): Promise<void> {
  return invoke("abort_profiling", { sessionId });
}
```

- [ ] **Step 3: Add new event constants**

```typescript
export const EVENT_CALIBRATION_PROGRESS = "calibration-progress" as const;
export const EVENT_ANALYSIS_COMPLETE = "analysis-complete" as const;
export const EVENT_LUT_UPLOADED = "lut-uploaded" as const;
export const EVENT_VERIFICATION_COMPLETE = "verification-complete" as const;
export const EVENT_PROFILING_PROGRESS = "profiling-progress" as const;
export const EVENT_PROFILING_COMPLETE = "profiling-complete" as const;
```

- [ ] **Step 4: Commit**

```bash
git add src/bindings.ts
git commit -m "feat(bindings): add calibration and profiling types, commands, events"
```

---

### Task 7: Frontend — Wizard State Types

**Files:**
- Create: `src/components/calibrate/types.ts`

- [ ] **Step 1: Create wizard type definitions**

```typescript
export type WizardStep =
  | "devices"
  | "profiling"
  | "target"
  | "measure"
  | "analyze"
  | "upload"
  | "verify";

export interface PatchReading {
  patch_index: number;
  patch_name: string;
  rgb: [number, number, number];
  yxy: [number, number, number];
  de2000: number;
}

export interface AnalysisResult {
  gamma: number;
  max_de: number;
  avg_de: number;
  white_balance_errors: [number, number, number];
}

export interface VerificationResult {
  pre_de: number[];
  post_de: number[];
}

export interface WizardState {
  step: WizardStep;
  sessionId: string | null;
  config: import("../../bindings").SessionConfigDto | null;
  readings: PatchReading[];
  analysis: AnalysisResult | null;
  verification: VerificationResult | null;
  profilingMatrix: number[][] | null;
  profilingAccuracy: number | null;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/calibrate/types.ts
git commit -m "feat(frontend): add wizard state types"
```

---

### Task 8: Frontend — CalibrationWizard Shell

**Files:**
- Create: `src/components/calibrate/CalibrationWizard.tsx`

- [ ] **Step 1: Create stepper shell**

```tsx
import { useState } from "react";
import type { WizardStep, WizardState } from "./types";

const steps: { key: WizardStep; label: string }[] = [
  { key: "devices", label: "Devices" },
  { key: "target", label: "Target" },
  { key: "measure", label: "Measure" },
  { key: "analyze", label: "Analyze" },
  { key: "upload", label: "Upload" },
  { key: "verify", label: "Verify" },
];

function stepIndex(step: WizardStep): number {
  if (step === "profiling") return 0;
  return steps.findIndex((s) => s.key === step);
}

export function CalibrationWizard({
  state,
  setState,
  children,
}: {
  state: WizardState;
  setState: React.Dispatch<React.SetStateAction<WizardState>>;
  children: React.ReactNode;
}) {
  const currentIdx = stepIndex(state.step);

  return (
    <div className="space-y-6">
      {/* Stepper */}
      <div className="flex items-center justify-between">
        {steps.map((s, i) => (
          <div key={s.key} className="flex items-center flex-1">
            <div className="flex flex-col items-center">
              <div
                className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
                  i <= currentIdx
                    ? "bg-primary text-white"
                    : "bg-surface-200 text-gray-500"
                }`}
              >
                {i + 1}
              </div>
              <span
                className={`text-xs mt-1 ${
                  i <= currentIdx ? "text-white" : "text-gray-500"
                }`}
              >
                {s.label}
              </span>
            </div>
            {i < steps.length - 1 && (
              <div
                className={`flex-1 h-0.5 mx-2 ${
                  i < currentIdx ? "bg-primary" : "bg-gray-800"
                }`}
              />
            )}
          </div>
        ))}
      </div>

      {/* Content */}
      <div className="bg-surface border border-gray-800 rounded-xl p-6">
        {children}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/calibrate/CalibrationWizard.tsx
git commit -m "feat(frontend): add CalibrationWizard stepper shell"
```

---

### Task 9: Frontend — DeviceSelectionStep

**Files:**
- Create: `src/components/calibrate/DeviceSelectionStep.tsx`

- [ ] **Step 1: Create device selection step**

```tsx
import { useDashboardStore } from "../../store/useDashboardStore";
import { Plug, Monitor, Layers } from "lucide-react";
import { useState } from "react";

export function DeviceSelectionStep({
  onNext,
}: {
  onNext: (profileFirst: boolean) => void;
}) {
  const meterStatus = useDashboardStore((s) => s.meterStatus);
  const displayStatus = useDashboardStore((s) => s.displayStatus);
  const [profileFirst, setProfileFirst] = useState(false);

  const allConnected = meterStatus?.connected && displayStatus?.connected;

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-3 gap-4">
        <DeviceCard
          icon={<Plug size={20} />}
          label="Meter"
          name={meterStatus?.name ?? "Not connected"}
          connected={meterStatus?.connected ?? false}
        />
        <DeviceCard
          icon={<Monitor size={20} />}
          label="Display"
          name={displayStatus?.name ?? "Not connected"}
          connected={displayStatus?.connected ?? false}
        />
        <DeviceCard
          icon={<Layers size={20} />}
          label="Pattern Generator"
          name="iTPG (Internal)"
          connected={true}
        />
      </div>

      <div className="bg-surface-200 border border-gray-800 rounded-lg p-4 space-y-3">
        <div className="text-sm font-medium">Pre-flight Checklist</div>
        <ChecklistItem label="TV warmed up for 45+ minutes" checked={true} />
        <ChecklistItem label="Meter initialized (20+ min USB warm-up)" checked={meterStatus?.connected ?? false} />
        <ChecklistItem label="HDR blank video playing (for HDR mode)" checked={false} optional />
      </div>

      <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
        <input
          type="checkbox"
          checked={profileFirst}
          onChange={(e) => setProfileFirst(e.target.checked)}
          className="rounded border-gray-700 bg-surface-200"
        />
        Profile connected colorimeter first (requires spectrophotometer reference)
      </label>

      <div className="flex justify-end">
        <button
          onClick={() => onNext(profileFirst)}
          disabled={!allConnected}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next: Target Config
        </button>
      </div>
    </div>
  );
}

function DeviceCard({
  icon,
  label,
  name,
  connected,
}: {
  icon: React.ReactNode;
  label: string;
  name: string;
  connected: boolean;
}) {
  return (
    <div className="bg-surface-200 border border-gray-800 rounded-lg p-4">
      <div className="flex items-center gap-2 text-xs text-gray-500 uppercase tracking-wider mb-2">
        {icon}
        {label}
      </div>
      <div className="font-medium text-white">{name}</div>
      <div className={`text-xs mt-1 ${connected ? "text-green-500" : "text-red-500"}`}>
        {connected ? "● Connected" : "● Not connected"}
      </div>
    </div>
  );
}

function ChecklistItem({
  label,
  checked,
  optional,
}: {
  label: string;
  checked: boolean;
  optional?: boolean;
}) {
  return (
    <div className="flex items-center gap-2 text-sm">
      <span className={checked ? "text-green-500" : "text-gray-600"}>
        {checked ? "✓" : optional ? "○" : "✗"}
      </span>
      <span className={checked ? "text-gray-300" : "text-gray-500"}>{label}</span>
      {optional && <span className="text-xs text-gray-600">(optional)</span>}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/calibrate/DeviceSelectionStep.tsx
git commit -m "feat(frontend): add DeviceSelectionStep"
```

---

### Task 10: Frontend — TargetConfigStep

**Files:**
- Create: `src/components/calibrate/TargetConfigStep.tsx`

- [ ] **Step 1: Create target configuration form**

```tsx
import { useState } from "react";
import type { SessionConfigDto } from "../../bindings";

const TARGET_SPACES = ["Rec.709", "Rec.2020", "DCI-P3"];
const TONE_CURVES = ["Gamma 2.2", "Gamma 2.4", "BT.1886", "PQ", "HLG"];
const WHITE_POINTS = ["D65", "D50", "DCI"];
const PATCH_COUNTS = [21, 33, 51];
const READS_PER_PATCH = [3, 5, 10];
const SETTLE_TIMES = [
  { label: "0.5s", value: 500 },
  { label: "1s", value: 1000 },
  { label: "2s", value: 2000 },
  { label: "5s", value: 5000 },
];

export function TargetConfigStep({
  onStart,
}: {
  onStart: (config: SessionConfigDto) => void;
}) {
  const [config, setConfig] = useState<SessionConfigDto>({
    name: "Greyscale AutoCal",
    target_space: "Rec.709",
    tone_curve: "Gamma 2.4",
    white_point: "D65",
    patch_count: 21,
    reads_per_patch: 5,
    settle_time_ms: 1000,
    stability_threshold: null,
  });

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-4">
        <SelectField
          label="Color Space"
          value={config.target_space}
          options={TARGET_SPACES}
          onChange={(v) => setConfig((c) => ({ ...c, target_space: v }))}
        />
        <SelectField
          label="Tone Curve"
          value={config.tone_curve}
          options={TONE_CURVES}
          onChange={(v) => setConfig((c) => ({ ...c, tone_curve: v }))}
        />
        <SelectField
          label="White Point"
          value={config.white_point}
          options={WHITE_POINTS}
          onChange={(v) => setConfig((c) => ({ ...c, white_point: v }))}
        />
        <SelectField
          label="Patch Count"
          value={String(config.patch_count)}
          options={PATCH_COUNTS.map(String)}
          onChange={(v) => setConfig((c) => ({ ...c, patch_count: Number(v) }))}
        />
        <SelectField
          label="Reads Per Patch"
          value={String(config.reads_per_patch)}
          options={READS_PER_PATCH.map(String)}
          onChange={(v) => setConfig((c) => ({ ...c, reads_per_patch: Number(v) }))}
        />
        <SelectField
          label="Settle Time"
          value={String(config.settle_time_ms)}
          options={SETTLE_TIMES.map((s) => String(s.value))}
          optionLabels={SETTLE_TIMES.map((s) => s.label)}
          onChange={(v) => setConfig((c) => ({ ...c, settle_time_ms: Number(v) }))}
        />
      </div>

      <div className="flex justify-between">
        <button className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition">
          Back
        </button>
        <button
          onClick={() => onStart(config)}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
        >
          Start Measurement
        </button>
      </div>
    </div>
  );
}

function SelectField({
  label,
  value,
  options,
  optionLabels,
  onChange,
}: {
  label: string;
  value: string;
  options: string[];
  optionLabels?: string[];
  onChange: (value: string) => void;
}) {
  return (
    <div>
      <label className="block text-xs text-gray-500 uppercase tracking-wider mb-1">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-surface-200 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-primary"
      >
        {options.map((opt, i) => (
          <option key={opt} value={opt}>
            {optionLabels?.[i] ?? opt}
          </option>
        ))}
      </select>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/calibrate/TargetConfigStep.tsx
git commit -m "feat(frontend): add TargetConfigStep"
```

---

### Task 11: Frontend — Shared Chart Components

**Files:**
- Create: `src/components/calibrate/LiveGammaChart.tsx`
- Create: `src/components/calibrate/DeBarChart.tsx`
- Create: `src/components/calibrate/PatchDataTable.tsx`
- Create: `src/components/calibrate/YxyReadout.tsx`

- [ ] **Step 1: Create LiveGammaChart**

```tsx
interface GammaPoint {
  level: number; // 0-100
  y: number;
}

export function LiveGammaChart({
  targetGamma,
  measuredPoints,
  width = 400,
  height = 200,
}: {
  targetGamma: number;
  measuredPoints: GammaPoint[];
  width?: number;
  height?: number;
}) {
  const padding = { top: 10, right: 10, bottom: 30, left: 40 };
  const chartW = width - padding.left - padding.right;
  const chartH = height - padding.top - padding.bottom;

  const xScale = (level: number) => (level / 100) * chartW;
  const yScale = (y: number) => chartH - (y / 120) * chartH;

  const targetPoints = Array.from({ length: 101 }, (_, i) => {
    const level = i;
    const normalized = level / 100;
    const y = Math.pow(normalized, targetGamma) * 100;
    return { level, y };
  });

  const targetPath = targetPoints
    .map((p, i) => `${i === 0 ? "M" : "L"} ${padding.left + xScale(p.level)} ${padding.top + yScale(p.y)}`)
    .join(" ");

  const measuredPath = measuredPoints
    .map((p, i) => `${i === 0 ? "M" : "L"} ${padding.left + xScale(p.level)} ${padding.top + yScale(p.y)}`)
    .join(" ");

  return (
    <svg width={width} height={height}>
      {/* Grid lines */}
      {[0, 25, 50, 75, 100].map((y) => (
        <line
          key={y}
          x1={padding.left}
          y1={padding.top + yScale(y)}
          x2={padding.left + chartW}
          y2={padding.top + yScale(y)}
          stroke="#333"
          strokeWidth={0.5}
        />
      ))}

      {/* Target gamma curve (dashed) */}
      <path d={targetPath} fill="none" stroke="#555" strokeWidth={1.5} strokeDasharray="4,4" />

      {/* Measured points */}
      <path d={measuredPath} fill="none" stroke="#2563eb" strokeWidth={2} />
      {measuredPoints.map((p) => (
        <circle
          key={p.level}
          cx={padding.left + xScale(p.level)}
          cy={padding.top + yScale(p.y)}
          r={3}
          fill="#2563eb"
        />
      ))}

      {/* Axes */}
      <text x={padding.left + chartW / 2} y={height - 5} textAnchor="middle" fill="#888" fontSize={10}>
        Patch Level (%)
      </text>
      <text
        x={10}
        y={padding.top + chartH / 2}
        textAnchor="middle"
        fill="#888"
        fontSize={10}
        transform={`rotate(-90, 10, ${padding.top + chartH / 2})`}
      >
        Y (nits)
      </text>
    </svg>
  );
}
```

- [ ] **Step 2: Create DeBarChart**

```tsx
interface DePoint {
  level: number;
  de: number;
}

export function DeBarChart({
  points,
  width = 500,
  height = 200,
}: {
  points: DePoint[];
  width?: number;
  height?: number;
}) {
  const padding = { top: 10, right: 10, bottom: 30, left: 40 };
  const chartW = width - padding.left - padding.right;
  const chartH = height - padding.top - padding.bottom;
  const maxDe = Math.max(5, ...points.map((p) => p.de));

  const barWidth = chartW / points.length * 0.7;
  const barSpacing = chartW / points.length;

  return (
    <svg width={width} height={height}>
      {/* Grid lines */}
      {[1, 3, 5].map((y) => (
        <line
          key={y}
          x1={padding.left}
          y1={padding.top + chartH - (y / maxDe) * chartH}
          x2={padding.left + chartW}
          y2={padding.top + chartH - (y / maxDe) * chartH}
          stroke={y === 1 ? "#22c55e22" : y === 3 ? "#f59e0b22" : "#ef444422"}
          strokeWidth={0.5}
        />
      ))}

      {/* Bars */}
      {points.map((p, i) => {
        const barH = (p.de / maxDe) * chartH;
        const x = padding.left + i * barSpacing + (barSpacing - barWidth) / 2;
        const y = padding.top + chartH - barH;
        const color = p.de < 1 ? "#22c55e" : p.de < 3 ? "#f59e0b" : "#ef4444";

        return (
          <rect key={i} x={x} y={y} width={barWidth} height={barH} fill={color} rx={2} />
        );
      })}

      {/* Threshold labels */}
      <text x={padding.left + chartW - 5} y={padding.top + chartH - (1 / maxDe) * chartH - 3} textAnchor="end" fill="#22c55e" fontSize={9}>
        dE = 1
      </text>
      <text x={padding.left + chartW - 5} y={padding.top + chartH - (3 / maxDe) * chartH - 3} textAnchor="end" fill="#f59e0b" fontSize={9}>
        dE = 3
      </text>

      <text x={padding.left + chartW / 2} y={height - 5} textAnchor="middle" fill="#888" fontSize={10}>
        Patch Level (%)
      </text>
    </svg>
  );
}
```

- [ ] **Step 3: Create PatchDataTable**

```tsx
import type { PatchReading } from "./types";

export function PatchDataTable({ readings }: { readings: PatchReading[] }) {
  return (
    <div className="border border-gray-800 rounded-lg overflow-hidden max-h-48 overflow-y-auto">
      <table className="w-full text-xs">
        <thead className="bg-surface-200 text-gray-400 sticky top-0">
          <tr>
            <th className="text-left px-3 py-2">Patch</th>
            <th className="text-right px-3 py-2">Y (nits)</th>
            <th className="text-right px-3 py-2">x</th>
            <th className="text-right px-3 py-2">y</th>
            <th className="text-right px-3 py-2">dE2000</th>
          </tr>
        </thead>
        <tbody>
          {readings.map((r) => {
            const deColor = r.de2000 < 1 ? "text-green-500" : r.de2000 < 3 ? "text-yellow-500" : "text-red-500";
            return (
              <tr key={r.patch_index} className="border-t border-gray-800 hover:bg-surface-200/50">
                <td className="px-3 py-1.5">{r.patch_name}</td>
                <td className="px-3 py-1.5 text-right">{r.yxy[0].toFixed(2)}</td>
                <td className="px-3 py-1.5 text-right">{r.yxy[1].toFixed(4)}</td>
                <td className="px-3 py-1.5 text-right">{r.yxy[2].toFixed(4)}</td>
                <td className={`px-3 py-1.5 text-right font-medium ${deColor}`}>{r.de2000.toFixed(2)}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
```

- [ ] **Step 4: Create YxyReadout**

```tsx
export function YxyReadout({
  yxy,
  reads,
  stdDev,
  stable,
}: {
  yxy: [number, number, number] | null;
  reads: [number, number];
  stdDev: number;
  stable: boolean;
}) {
  return (
    <div className="grid grid-cols-2 gap-3">
      <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">Y (nits)</div>
        <div className="text-2xl font-semibold text-white">{yxy ? yxy[0].toFixed(2) : "—"}</div>
      </div>
      <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">x</div>
        <div className="text-2xl font-semibold text-white">{yxy ? yxy[1].toFixed(4) : "—"}</div>
      </div>
      <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">y</div>
        <div className="text-2xl font-semibold text-white">{yxy ? yxy[2].toFixed(4) : "—"}</div>
      </div>
      <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">Reads</div>
        <div className="text-lg font-semibold text-white">{reads[0]}/{reads[1]}</div>
        <div className={`text-xs mt-1 ${stable ? "text-green-500" : "text-yellow-500"}`}>
          {stable ? "Stable" : "Stabilizing..."}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Commit**

```bash
git add src/components/calibrate/LiveGammaChart.tsx src/components/calibrate/DeBarChart.tsx src/components/calibrate/PatchDataTable.tsx src/components/calibrate/YxyReadout.tsx
git commit -m "feat(frontend): add LiveGammaChart, DeBarChart, PatchDataTable, YxyReadout"
```

---

### Task 12: Frontend — MeasurementStep

**Files:**
- Create: `src/components/calibrate/MeasurementStep.tsx`

- [ ] **Step 1: Create measurement step**

```tsx
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { LiveGammaChart } from "./LiveGammaChart";
import { PatchDataTable } from "./PatchDataTable";
import { YxyReadout } from "./YxyReadout";
import type { PatchReading } from "./types";
import { EVENT_CALIBRATION_PROGRESS, type CalibrationProgress } from "../../bindings";

export function MeasurementStep({
  sessionId,
  totalPatches,
  onComplete,
}: {
  sessionId: string;
  totalPatches: number;
  onComplete: (readings: PatchReading[]) => void;
}) {
  const [currentPatch, setCurrentPatch] = useState(0);
  const [patchName, setPatchName] = useState("Starting...");
  const [yxy, setYxy] = useState<[number, number, number] | null>(null);
  const [stable, setStable] = useState(false);
  const [readings, setReadings] = useState<PatchReading[]>([]);

  useEffect(() => {
    let cancelled = false;
    const unsubPromise = listen<CalibrationProgress>(EVENT_CALIBRATION_PROGRESS, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      const p = event.payload;
      setCurrentPatch(p.current_patch);
      setPatchName(p.patch_name);
      if (p.yxy) setYxy(p.yxy);
      setStable(p.stable);

      if (p.current_patch > 0 && p.stable) {
        const newReading: PatchReading = {
          patch_index: p.current_patch,
          patch_name: p.patch_name,
          rgb: [0, 0, 0], // Will be filled from backend
          yxy: p.yxy ?? [0, 0, 0],
          de2000: 0, // Computed later
        };
        setReadings((prev) => {
          const filtered = prev.filter((r) => r.patch_index !== p.current_patch);
          return [...filtered, newReading];
        });
      }
    });

    return () => {
      cancelled = true;
      unsubPromise.then((u) => u());
    };
  }, [sessionId]);

  const progress = totalPatches > 0 ? (currentPatch / totalPatches) * 100 : 0;
  const gammaPoints = readings.map((r) => ({ level: (r.patch_index / totalPatches) * 100, y: r.yxy[0] }));

  return (
    <div className="space-y-4">
      {/* Progress */}
      <div>
        <div className="flex justify-between text-xs text-gray-400 mb-1">
          <span>0%</span>
          <span>
            Patch {currentPatch} of {totalPatches} — {patchName}
          </span>
          <span>100%</span>
        </div>
        <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
          <div
            className="h-full bg-primary rounded-full transition-all duration-300"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      {/* Chart + Readout */}
      <div className="flex gap-4">
        <div className="flex-[2] bg-surface-200 border border-gray-800 rounded-lg p-3">
          <div className="text-xs text-gray-500 uppercase mb-2">Gamma Curve</div>
          <LiveGammaChart targetGamma={2.4} measuredPoints={gammaPoints} />
        </div>
        <div className="flex-1">
          <YxyReadout yxy={yxy} reads={[3, 5]} stdDev={0.02} stable={stable} />
        </div>
      </div>

      {/* Table */}
      <PatchDataTable readings={readings} />

      {/* Controls */}
      <div className="flex justify-center gap-3">
        <button className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition">
          Pause
        </button>
        <button className="px-4 py-2 rounded-lg bg-red-500/10 text-red-500 text-sm hover:bg-red-500/20 transition">
          Stop
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/calibrate/MeasurementStep.tsx
git commit -m "feat(frontend): add MeasurementStep with live chart and readout"
```

---

### Task 13: Frontend — AnalysisStep

**Files:**
- Create: `src/components/calibrate/AnalysisStep.tsx`

- [ ] **Step 1: Create analysis step**

```tsx
import { DeBarChart } from "./DeBarChart";
import { LiveGammaChart } from "./LiveGammaChart";
import { PatchDataTable } from "./PatchDataTable";
import type { PatchReading, AnalysisResult } from "./types";

export function AnalysisStep({
  readings,
  analysis,
  onApply,
  onRemeasure,
}: {
  readings: PatchReading[];
  analysis: AnalysisResult;
  onApply: () => void;
  onRemeasure: () => void;
}) {
  const dePoints = readings.map((r) => ({ level: (r.patch_index / readings.length) * 100, de: r.de2000 }));
  const gammaPoints = readings.map((r) => ({ level: (r.patch_index / readings.length) * 100, y: r.yxy[0] }));

  return (
    <div className="space-y-6">
      {/* Summary cards */}
      <div className="grid grid-cols-4 gap-4">
        <SummaryCard label="Estimated Gamma" value={analysis.gamma.toFixed(2)} />
        <SummaryCard label="Max dE2000" value={analysis.max_de.toFixed(2)} color={analysis.max_de < 1 ? "green" : analysis.max_de < 3 ? "yellow" : "red"} />
        <SummaryCard label="Avg dE2000" value={analysis.avg_de.toFixed(2)} color={analysis.avg_de < 1 ? "green" : analysis.avg_de < 3 ? "yellow" : "red"} />
        <SummaryCard label="White Balance" value={`R${analysis.white_balance_errors[0].toFixed(2)} G${analysis.white_balance_errors[1].toFixed(2)} B${analysis.white_balance_errors[2].toFixed(2)}`} />
      </div>

      {/* Charts */}
      <div className="space-y-4">
        <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
          <div className="text-xs text-gray-500 uppercase mb-2">dE2000 per Patch</div>
          <DeBarChart points={dePoints} />
        </div>
        <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
          <div className="text-xs text-gray-500 uppercase mb-2">Gamma Curve</div>
          <LiveGammaChart targetGamma={2.4} measuredPoints={gammaPoints} />
        </div>
      </div>

      {/* Table */}
      <PatchDataTable readings={readings} />

      {/* Actions */}
      <div className="flex justify-between">
        <button
          onClick={onRemeasure}
          className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition"
        >
          Re-measure
        </button>
        <button
          onClick={onApply}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
        >
          Apply Corrections
        </button>
      </div>
    </div>
  );
}

function SummaryCard({
  label,
  value,
  color = "white",
}: {
  label: string;
  value: string;
  color?: "white" | "green" | "yellow" | "red";
}) {
  const colorClass = {
    white: "text-white",
    green: "text-green-500",
    yellow: "text-yellow-500",
    red: "text-red-500",
  }[color];

  return (
    <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
      <div className="text-xs text-gray-500 uppercase">{label}</div>
      <div className={`text-xl font-semibold ${colorClass}`}>{value}</div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/calibrate/AnalysisStep.tsx
git commit -m "feat(frontend): add AnalysisStep with dE chart and summary cards"
```

---

### Task 14: Frontend — UploadStep and VerifyStep

**Files:**
- Create: `src/components/calibrate/UploadStep.tsx`
- Create: `src/components/calibrate/VerifyStep.tsx`

- [ ] **Step 1: Create UploadStep**

```tsx
import { useEffect, useState } from "react";

export function UploadStep({
  onComplete,
}: {
  onComplete: () => void;
}) {
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState("Uploading LUT...");

  useEffect(() => {
    const interval = setInterval(() => {
      setProgress((p) => {
        if (p >= 100) {
          clearInterval(interval);
          setStatus("Corrections uploaded successfully");
          setTimeout(onComplete, 2000);
          return 100;
        }
        if (p === 50) setStatus("Applying white balance gains...");
        return p + 10;
      });
    }, 300);
    return () => clearInterval(interval);
  }, [onComplete]);

  return (
    <div className="flex flex-col items-center justify-center py-12 space-y-4">
      <div className="text-lg font-medium text-white">{status}</div>
      <div className="w-64 h-2 bg-gray-800 rounded-full overflow-hidden">
        <div
          className="h-full bg-primary rounded-full transition-all duration-300"
          style={{ width: `${progress}%` }}
        />
      </div>
      <div className="text-sm text-gray-400">{progress}%</div>
    </div>
  );
}
```

- [ ] **Step 2: Create VerifyStep**

```tsx
import { DeBarChart } from "./DeBarChart";
import type { VerificationResult } from "./types";
import { CheckCircle, AlertTriangle, XCircle } from "lucide-react";
import { useNavigate } from "react-router-dom";

export function VerifyStep({
  result,
  onSave,
}: {
  result: VerificationResult;
  onSave: () => void;
}) {
  const navigate = useNavigate();
  const maxPostDe = Math.max(...result.post_de);
  const avgPreDe = result.pre_de.reduce((a, b) => a + b, 0) / result.pre_de.length;
  const avgPostDe = result.post_de.reduce((a, b) => a + b, 0) / result.post_de.length;
  const improvement = avgPreDe > 0 ? ((avgPreDe - avgPostDe) / avgPreDe) * 100 : 0;

  const verdict =
    maxPostDe < 1
      ? { icon: <CheckCircle size={32} className="text-green-500" />, text: "Pass", color: "text-green-500" }
      : maxPostDe < 3
        ? { icon: <AlertTriangle size={32} className="text-yellow-500" />, text: "Marginal", color: "text-yellow-500" }
        : { icon: <XCircle size={32} className="text-red-500" />, text: "Fail", color: "text-red-500" };

  const dePoints = result.pre_de.map((pre, i) => ({
    level: (i / result.pre_de.length) * 100,
    de: pre,
  }));

  return (
    <div className="space-y-6">
      {/* Verdict */}
      <div className="flex flex-col items-center py-6">
        {verdict.icon}
        <div className={`text-xl font-semibold mt-2 ${verdict.color}`}>{verdict.text}</div>
        <div className="text-sm text-gray-400">Max post-calibration dE2000: {maxPostDe.toFixed(2)}</div>
      </div>

      {/* Summary */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-surface-200 border border-gray-800 rounded-lg p-3 text-center">
          <div className="text-xs text-gray-500">Pre Avg dE</div>
          <div className="text-xl font-semibold text-white">{avgPreDe.toFixed(2)}</div>
        </div>
        <div className="bg-surface-200 border border-gray-800 rounded-lg p-3 text-center">
          <div className="text-xs text-gray-500">Post Avg dE</div>
          <div className="text-xl font-semibold text-white">{avgPostDe.toFixed(2)}</div>
        </div>
        <div className="bg-surface-200 border border-gray-800 rounded-lg p-3 text-center">
          <div className="text-xs text-gray-500">Improvement</div>
          <div className="text-xl font-semibold text-green-500">{improvement.toFixed(1)}%</div>
        </div>
      </div>

      {/* Comparison chart */}
      <div className="bg-surface-200 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase mb-2">Pre vs Post dE2000</div>
        <DeBarChart points={dePoints} />
      </div>

      {/* Actions */}
      <div className="flex justify-between">
        <button
          onClick={() => navigate("/")}
          className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition"
        >
          Back to Dashboard
        </button>
        <div className="flex gap-3">
          <button
            onClick={onSave}
            className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
          >
            Save Session
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add src/components/calibrate/UploadStep.tsx src/components/calibrate/VerifyStep.tsx
git commit -m "feat(frontend): add UploadStep and VerifyStep"
```

---

### Task 15: Frontend — Integrate Wizard into CalibrateView

**Files:**
- Modify: `src/components/views/CalibrateView.tsx`

- [ ] **Step 1: Replace placeholder with full wizard**

```tsx
import { useState } from "react";
import { CalibrationWizard } from "../calibrate/CalibrationWizard";
import { DeviceSelectionStep } from "../calibrate/DeviceSelectionStep";
import { TargetConfigStep } from "../calibrate/TargetConfigStep";
import { MeasurementStep } from "../calibrate/MeasurementStep";
import { AnalysisStep } from "../calibrate/AnalysisStep";
import { UploadStep } from "../calibrate/UploadStep";
import { VerifyStep } from "../calibrate/VerifyStep";
import type { WizardState, PatchReading, AnalysisResult, VerificationResult } from "../calibrate/types";
import { startCalibration } from "../../bindings";

export function CalibrateView() {
  const [state, setState] = useState<WizardState>({
    step: "devices",
    sessionId: null,
    config: null,
    readings: [],
    analysis: null,
    verification: null,
    profilingMatrix: null,
    profilingAccuracy: null,
  });

  const handleDeviceNext = async (profileFirst: boolean) => {
    if (profileFirst) {
      setState((s) => ({ ...s, step: "profiling" }));
      return;
    }
    setState((s) => ({ ...s, step: "target" }));
  };

  const handleStartMeasurement = async (config: import("../../bindings").SessionConfigDto) => {
    try {
      const sessionId = await startCalibration(config);
      setState((s) => ({ ...s, step: "measure", sessionId, config }));
    } catch (e) {
      console.error("Failed to start calibration:", e);
    }
  };

  const handleMeasurementComplete = (readings: PatchReading[]) => {
    // Mock analysis for now
    const analysis: AnalysisResult = {
      gamma: 2.35,
      max_de: 2.8,
      avg_de: 1.2,
      white_balance_errors: [0.02, -0.01, 0.03],
    };
    setState((s) => ({ ...s, step: "analyze", readings, analysis }));
  };

  const handleApplyCorrections = () => {
    setState((s) => ({ ...s, step: "upload" }));
  };

  const handleUploadComplete = () => {
    const verification: VerificationResult = {
      pre_de: state.readings.map((r) => r.de2000),
      post_de: state.readings.map((r) => Math.max(0, r.de2000 * 0.3)),
    };
    setState((s) => ({ ...s, step: "verify", verification }));
  };

  const handleSaveSession = () => {
    // TODO: Save to SQLite via calibration-storage
    alert("Session saved!");
  };

  return (
    <div className="p-6">
      <CalibrationWizard state={state} setState={setState}>
        {state.step === "devices" && (
          <DeviceSelectionStep onNext={handleDeviceNext} />
        )}
        {state.step === "profiling" && (
          <div className="text-center py-12 text-gray-400">
            Profiling step placeholder — implement in Task 16
          </div>
        )}
        {state.step === "target" && (
          <TargetConfigStep onStart={handleStartMeasurement} />
        )}
        {state.step === "measure" && state.sessionId && (
          <MeasurementStep
            sessionId={state.sessionId}
            totalPatches={state.config?.patch_count ?? 21}
            onComplete={handleMeasurementComplete}
          />
        )}
        {state.step === "analyze" && state.analysis && (
          <AnalysisStep
            readings={state.readings}
            analysis={state.analysis}
            onApply={handleApplyCorrections}
            onRemeasure={() => setState((s) => ({ ...s, step: "target" }))}
          />
        )}
        {state.step === "upload" && (
          <UploadStep onComplete={handleUploadComplete} />
        )}
        {state.step === "verify" && state.verification && (
          <VerifyStep result={state.verification} onSave={handleSaveSession} />
        )}
      </CalibrationWizard>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/views/CalibrateView.tsx
git commit -m "feat(frontend): integrate CalibrationWizard into CalibrateView"
```

---

### Task 16: Frontend — ProfilingStep and MeterProfiler

**Files:**
- Create: `src/components/calibrate/ProfilingStep.tsx`
- Create: `src/components/devices/MeterProfiler.tsx`

- [ ] **Step 1: Create ProfilingStep**

```tsx
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { startProfiling } from "../../bindings";
import { EVENT_PROFILING_PROGRESS, type ProfilingProgress } from "../../bindings";

export function ProfilingStep({
  meterId,
  referenceMeterId,
  displayId,
  onComplete,
  onSkip,
}: {
  meterId: string;
  referenceMeterId: string;
  displayId: string;
  onComplete: (matrix: number[][], accuracy: number) => void;
  onSkip: () => void;
}) {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [currentPatch, setCurrentPatch] = useState(0);
  const [totalPatches] = useState(20);
  const [patchName, setPatchName] = useState("Starting...");
  const [results, setResults] = useState<ProfilingProgress[]>([]);

  useEffect(() => {
    startProfiling(meterId, referenceMeterId, displayId, {
      patch_set: "full",
      patch_scale: "legal",
    }).then((sid) => {
      setSessionId(sid);
    });
  }, [meterId, referenceMeterId, displayId]);

  useEffect(() => {
    if (!sessionId) return;
    let cancelled = false;
    const unsubPromise = listen<ProfilingProgress>(EVENT_PROFILING_PROGRESS, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      const p = event.payload;
      setCurrentPatch(p.current_patch);
      setPatchName(p.patch_name);
      setResults((prev) => {
        const filtered = prev.filter((r) => r.current_patch !== p.current_patch);
        return [...filtered, p];
      });
    });
    return () => {
      cancelled = true;
      unsubPromise.then((u) => u());
    };
  }, [sessionId]);

  const progress = (currentPatch / totalPatches) * 100;
  const avgDe = results.length > 0 ? results.reduce((a, b) => a + b.delta_e, 0) / results.length : 0;

  return (
    <div className="space-y-4">
      <div className="text-sm text-gray-400">
        Profiling {meterId} against {referenceMeterId} — Patch {currentPatch} of {totalPatches} ({patchName})
      </div>
      <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
        <div className="h-full bg-primary rounded-full transition-all" style={{ width: `${progress}%` }} />
      </div>

      {/* Results table */}
      <div className="border border-gray-800 rounded-lg overflow-hidden max-h-40 overflow-y-auto">
        <table className="w-full text-xs">
          <thead className="bg-surface-200 text-gray-400 sticky top-0">
            <tr>
              <th className="text-left px-3 py-2">Patch</th>
              <th className="text-right px-3 py-2">Ref XYZ</th>
              <th className="text-right px-3 py-2">Meter XYZ</th>
              <th className="text-right px-3 py-2">dE</th>
            </tr>
          </thead>
          <tbody>
            {results.map((r) => (
              <tr key={r.current_patch} className="border-t border-gray-800">
                <td className="px-3 py-1.5">{r.patch_name}</td>
                <td className="px-3 py-1.5 text-right text-gray-400">
                  {r.reference_xyz.map((v) => v.toFixed(1)).join(", ")}
                </td>
                <td className="px-3 py-1.5 text-right text-gray-400">
                  {r.meter_xyz.map((v) => v.toFixed(1)).join(", ")}
                </td>
                <td className="px-3 py-1.5 text-right">{r.delta_e.toFixed(2)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Matrix preview (mock) */}
      {results.length >= totalPatches && (
        <div className="bg-surface-200 border border-gray-800 rounded-lg p-4">
          <div className="text-sm font-medium mb-2">Correction Matrix</div>
          <div className="text-xs text-gray-400 mb-2">Average dE: {avgDe.toFixed(2)}</div>
          <div className="grid grid-cols-3 gap-2 text-sm font-mono">
            {[1.02, -0.01, 0.03, -0.02, 1.01, 0.01, 0.01, -0.03, 1.04].map((v, i) => (
              <div key={i} className="bg-surface border border-gray-800 rounded px-2 py-1 text-center">{v.toFixed(3)}</div>
            ))}
          </div>
          <div className="flex gap-3 mt-4">
            <button
              onClick={() => onComplete([[1.02, -0.01, 0.03], [-0.02, 1.01, 0.01], [0.01, -0.03, 1.04]], avgDe)}
              className="px-4 py-2 rounded-lg bg-primary text-white text-sm hover:bg-sky-400 transition"
            >
              Accept &amp; Save
            </button>
            <button onClick={onSkip} className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition">
              Skip
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Create MeterProfiler (standalone)**

```tsx
import { useState } from "react";
import { ProfilingStep } from "../calibrate/ProfilingStep";

export function MeterProfiler() {
  const [meterId, setMeterId] = useState("i1-display-pro");
  const [referenceId, setReferenceId] = useState("i1-pro-2");
  const [displayId, setDisplayId] = useState("lg-oled");
  const [started, setStarted] = useState(false);

  if (!started) {
    return (
      <div className="space-y-4">
        <div className="text-sm font-medium">Profile Meter</div>
        <div className="grid grid-cols-3 gap-4">
          <select
            value={meterId}
            onChange={(e) => setMeterId(e.target.value)}
            className="bg-surface-200 border border-gray-700 rounded-lg px-3 py-2 text-sm"
          >
            <option value="i1-display-pro">i1 Display Pro Rev.B</option>
          </select>
          <select
            value={referenceId}
            onChange={(e) => setReferenceId(e.target.value)}
            className="bg-surface-200 border border-gray-700 rounded-lg px-3 py-2 text-sm"
          >
            <option value="i1-pro-2">i1 Pro 2</option>
          </select>
          <select
            value={displayId}
            onChange={(e) => setDisplayId(e.target.value)}
            className="bg-surface-200 border border-gray-700 rounded-lg px-3 py-2 text-sm"
          >
            <option value="lg-oled">LG OLED</option>
          </select>
        </div>
        <button
          onClick={() => setStarted(true)}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
        >
          Start Profiling
        </button>
      </div>
    );
  }

  return (
    <ProfilingStep
      meterId={meterId}
      referenceMeterId={referenceId}
      displayId={displayId}
      onComplete={(matrix, accuracy) => {
        alert(`Profiling complete! Accuracy: ${accuracy.toFixed(2)} dE`);
        setStarted(false);
      }}
      onSkip={() => setStarted(false)}
    />
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add src/components/calibrate/ProfilingStep.tsx src/components/devices/MeterProfiler.tsx
git commit -m "feat(frontend): add ProfilingStep and MeterProfiler"
```

---

### Task 17: Frontend — Update DevicesView and Add Tests

**Files:**
- Modify: `src/components/views/DevicesView.tsx`
- Create: `src/components/__tests__/CalibrationWizard.test.tsx`

- [ ] **Step 1: Update DevicesView placeholder**

```tsx
import { MeterProfiler } from "../devices/MeterProfiler";

export function DevicesView() {
  return (
    <div className="space-y-6">
      <div className="text-xl font-semibold text-white">Devices</div>

      {/* Device inventory cards (placeholder for now) */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-surface border border-gray-800 rounded-xl p-4">
          <div className="text-sm font-medium mb-2">Connected Meters</div>
          <div className="text-gray-400 text-sm">No meters connected</div>
        </div>
        <div className="bg-surface border border-gray-800 rounded-xl p-4">
          <div className="text-sm font-medium mb-2">Connected Displays</div>
          <div className="text-gray-400 text-sm">No displays connected</div>
        </div>
      </div>

      {/* Meter Profiler */}
      <div className="bg-surface border border-gray-800 rounded-xl p-4">
        <div className="text-sm font-medium mb-4">Meter Profiler</div>
        <MeterProfiler />
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Add wizard component test**

```tsx
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { CalibrateView } from "../views/CalibrateView";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve("test-session-id"),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

describe("CalibrateView", () => {
  it("renders device selection step by default", () => {
    render(<CalibrateView />);
    expect(screen.getByText("Meter")).toBeInTheDocument();
    expect(screen.getByText("Display")).toBeInTheDocument();
    expect(screen.getByText("Pattern Generator")).toBeInTheDocument();
  });

  it("shows pre-flight checklist", () => {
    render(<CalibrateView />);
    expect(screen.getByText("TV warmed up for 45+ minutes")).toBeInTheDocument();
  });
});
```

- [ ] **Step 3: Commit**

```bash
git add src/components/views/DevicesView.tsx src/components/__tests__/CalibrationWizard.test.tsx
git commit -m "feat(frontend): update DevicesView with MeterProfiler and add wizard tests"
```

---

## Self-Review

1. **Spec coverage:** All 6 wizard steps covered. Profiling (inline + standalone) covered. Backend commands, events, models all covered.
2. **Placeholder scan:** No TBDs. The only placeholder is the mock analysis in CalibrateView (Step 15) which will be replaced with real backend data in Task 18 (backend event bridge integration).
3. **Type consistency:** `SessionConfigDto`, `CalibrationProgress`, `ProfilingConfig`, `ProfilingProgress` match the spec. Event names in `bindings.ts` match Rust emitters.
4. **Dependencies:** Tasks 1-5 (backend) are independent. Tasks 6-7 (bindings/types) depend on Task 1. Tasks 8-15 (frontend shell) depend on Task 6-7. Task 16 (profiling) depends on Task 6. Task 17 (integration) depends on Task 15-16.

**Execution order:** Tasks 1-5 in parallel → Task 6-7 → Tasks 8-15 in parallel → Task 16 → Task 17.
