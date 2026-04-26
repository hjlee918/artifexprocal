# Phase 4b: Calibration Wizard Design Spec

**Date:** 2026-04-26
**Status:** Approved
**Depends on:** Phase 4a (Dashboard Shell), Phase 3a (Calibration Engine), Phase 3b/c (Device Drivers)

---

## 1. Goal

Build the step-by-step greyscale AutoCal wizard that replaces the `CalibrateView` placeholder. The wizard guides the user through device selection, optional meter profiling, target configuration, live measurement with real-time feedback, post-calibration analysis, LUT upload, and post-calibration verification.

---

## 2. Architecture Overview

The wizard is a single-page React component (`CalibrateView`) that manages its own local step state. It communicates with the backend through:

- **Existing commands:** `connect_meter`, `connect_display`, `get_app_state`, `get_device_inventory`
- **New commands:** `start_calibration`, `abort_calibration`, `start_profiling`, `abort_profiling`
- **Existing events:** `device-status-changed`, `calibration-state-changed`, `error-occurred`
- **New events:** `calibration-progress`, `patch-complete`, `analysis-complete`, `lut-uploaded`, `verification-complete`, `profiling-progress`, `profiling-complete`

The backend wraps `GreyscaleAutoCalFlow::run_sync()` in `tokio::task::spawn_blocking` and bridges `CalibrationEvent` from the engine's `EventChannel` into Tauri events via `AppHandle::emit`.

---

## 3. Wizard Flow (6 Steps)

### Step 1: Device Selection
- Auto-populates connected meter, display, and pattern generator from `CalibrationService` state
- Inline "Change..." buttons open a device picker (reuses DevicesView inventory UI)
- Pre-flight checklist: TV warmed up, meter initialized, HDR blank video playing (for HDR mode)
- **NEW:** "Profile connected colorimeter first?" checkbox — if checked, inserts profiling flow before measurement
- Picture mode selection on the Display card (SDR/HDR10/Dolby Vision)
- "Next: Target Config" button

### Step 1a: Meter Profiling (optional inline)
- Only shown if a colorimeter (not spectro) is selected AND "Profile first" is checked
- Prompt: "Connect your spectrophotometer (i1 Pro 2) as reference"
- Displays both meters side-by-side with status dots
- 18-20 patch sequence:
  - R, G, B primaries (100% saturation)
  - C, M, Y secondaries (100% saturation)
  - Near-R, Near-G, Near-B (80% saturation)
  - White (100%), Black (0%), 50% Gray
  - Optional: skin tones, pure gray steps
- Each patch: measure with reference → measure with colorimeter → compute per-patch dE
- Live measurement UI reuses Step 3 components but patch labels are profiling-specific ("Primary Red", "Secondary Cyan", etc.)
- After all patches: generate 3x3 correction matrix, show matrix preview + estimated accuracy (average dE)
- User actions: **Accept & Save** (stores `.ccmx` + applies to current session), **Re-measure** (repeat), **Skip** (discard)

### Step 2: Target Configuration
- Color space: Rec.709 / Rec.2020 / DCI-P3
- Tone curve: Gamma 2.2 / 2.4 / BT.1886 / PQ (ST.2084) / HLG
- White point: D65 / D50 / DCI
- Patch count: 21 / 33 / 51 for greyscale ramp
- Reads per patch: 3 / 5 / 10
- Settle time: 0.5s / 1s / 2s / 5s
- Stability threshold: Auto / Manual override
- Correction matrix selection (if any saved `.ccmx` files exist, dropdown to pick)
- "Next: Start Measurement" button

### Step 3: Measure (Pre-Calibration)
- Progress bar + ETA (based on remaining patches × settle time × reads per patch)
- **Hero visualization:** SVG-based gamma curve chart (target dashed line + measured points connected by line). New point animates in as each patch completes.
- **Live readout:** Big numeric Yxy (Y in nits, x, y). Updates on every reading event.
- **Reading stats:** Read count (e.g. 3/5), std dev, stability flag (green = stable, orange = waiting)
- **Completed patch table:** Scrollable table with patch name, Y, x, y, dE2000. Color-coded: green <1.0, yellow 1.0-3.0, red >3.0
- **Controls:** Pause (finishes current patch then pauses), Stop (aborts immediately), Resume
- Backend: runs `GreyscaleAutoCalFlow::run_sync()` with pre-calibration flag

### Step 4: Analyze
- **Hero visualization:** dE2000 bar chart per patch (x-axis = patch level 0-100%, y-axis = dE2000)
- **Secondary:** Gamma curve (measured vs target) — smaller chart below the dE chart
- **Summary card (big numbers):**
  - Estimated gamma
  - Max dE2000
  - Average dE2000
  - White balance errors (R/G/B delta from neutral)
- **Patch data table:** Same as Step 3 table but full 21/33/51 rows
- **Action buttons:**
  - "Apply Corrections" → Step 5
  - "Re-measure" → back to Step 3 (preserves device config)
  - "Cancel" → aborts and returns to Step 1

### Step 5: Upload
- LUT upload progress bar (0-100%)
- White balance gain application status
- Display confirmation: "Corrections uploaded successfully"
- Auto-advance to Step 6 after 2 seconds
- Error handling: if upload fails, show error + "Retry" or "Cancel"

### Step 6: Verify (Post-Calibration)
- Re-measures same greyscale patches with new LUT active
- **Side-by-side comparison:** Pre dE vs Post dE bar chart (two bars per patch)
- **Pass/fail verdict:**
  - Green check: max dE < 1.0
  - Yellow warning: max dE < 3.0
  - Red cross: max dE >= 3.0
- **Summary:** Pre avg dE, Post avg dE, improvement percentage
- **Actions:**
  - "Save Session" → stores readings + results to SQLite (via calibration-storage)
  - "Export Report" → placeholder for Phase 9 (PDF/HTML report generation)
  - "New Calibration" → resets to Step 1
  - "Back to Dashboard" → navigates to "/"

---

## 4. Standalone Meter Profiler (Devices Page)

Accessible from `DevicesView` as a "Profile Meter" card/button.

**Flow:**
1. Select colorimeter to profile (dropdown from device inventory)
2. Select reference spectrophotometer (dropdown)
3. Display selection (for patch generation — iTPG or PGenerator)
4. Configure: patch set (Full/Quick), patch scale (Legal/Full)
5. Measure: same live UI as wizard Step 3 but patch labels are profiling-specific
6. Results: correction matrix preview (3x3 grid), per-patch dE table, accuracy estimate
7. Save: file picker for `.ccmx` output + optional name/description
8. Apply: set as active profile for the selected colorimeter

---

## 5. Backend Changes

### 5.1 New IPC Models

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

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct CalibrationProgress {
    pub session_id: String,
    pub current_patch: usize,
    pub total_patches: usize,
    pub patch_name: String,
    pub yxy: Option<(f64, f64, f64)>,
    pub stable: bool,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct ProfilingConfig {
    pub patch_set: String, // "full" or "quick"
    pub patch_scale: String, // "legal" or "full"
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

### 5.2 New Commands

```rust
#[tauri::command]
#[specta::specta]
pub fn start_calibration(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    config: SessionConfigDto,
) -> Result<String, String>;

#[tauri::command]
#[specta::specta]
pub fn abort_calibration(
    service: State<'_, CalibrationService>,
    session_id: String,
) -> Result<(), String>;

#[tauri::command]
#[specta::specta]
pub fn start_profiling(
    app: AppHandle,
    service: State<'_, CalibrationService>,
    meter_id: String,
    reference_meter_id: String,
    display_id: String,
    config: ProfilingConfig,
) -> Result<String, String>;

#[tauri::command]
#[specta::specta]
pub fn abort_profiling(
    service: State<'_, CalibrationService>,
    session_id: String,
) -> Result<(), String>;
```

### 5.3 New Events

```rust
pub fn emit_calibration_progress(
    app: &AppHandle,
    session_id: String,
    current_patch: usize,
    total_patches: usize,
    patch_name: String,
    yxy: Option<(f64, f64, f64)>,
    stable: bool,
);

pub fn emit_analysis_complete(
    app: &AppHandle,
    session_id: String,
    gamma: f64,
    max_de: f64,
    avg_de: f64,
    white_balance_errors: Vec<f64>,
);

pub fn emit_lut_uploaded(app: &AppHandle, session_id: String);

pub fn emit_verification_complete(
    app: &AppHandle,
    session_id: String,
    pre_de: Vec<f64>,
    post_de: Vec<f64>,
);

pub fn emit_profiling_progress(
    app: &AppHandle,
    session_id: String,
    current_patch: usize,
    total_patches: usize,
    patch_name: String,
    reference_xyz: (f64, f64, f64),
    meter_xyz: (f64, f64, f64),
    delta_e: f64,
);

pub fn emit_profiling_complete(
    app: &AppHandle,
    session_id: String,
    correction_matrix: [[f64; 3]; 3],
    accuracy_estimate: f64,
);
```

### 5.4 CalibrationService Updates

The `CalibrationService` needs to be extended to:
- Track active calibration session ID
- Store `SessionConfig` and `GreyscaleAutoCalFlow` state
- Spawn the calibration in a blocking thread
- Bridge `EventChannel` events to Tauri events via `AppHandle`
- Support profiling mode with two meters + display

```rust
pub struct CalibrationService {
    // existing fields: meter, meter_info, display, display_info, state, use_mocks
    active_session: Arc<Mutex<Option<CalibrationSession>>>,
}

struct CalibrationSession {
    session_id: String,
    config: SessionConfig,
    flow: GreyscaleAutoCalFlow,
    pre_readings: Vec<(RGB, XYZ)>,
}
```

---

## 6. Frontend Component Breakdown

| Component | File | Responsibility |
|-----------|------|---------------|
| `CalibrateView` | `src/components/views/CalibrateView.tsx` | Root wizard container, manages step state |
| `CalibrationWizard` | `src/components/calibrate/CalibrationWizard.tsx` | Stepper layout + next/back navigation |
| `DeviceSelectionStep` | `src/components/calibrate/DeviceSelectionStep.tsx` | Step 1: device cards + pre-flight checklist |
| `ProfilingStep` | `src/components/calibrate/ProfilingStep.tsx` | Step 1a: inline meter profiling |
| `TargetConfigStep` | `src/components/calibrate/TargetConfigStep.tsx` | Step 2: target settings form |
| `MeasurementStep` | `src/components/calibrate/MeasurementStep.tsx` | Step 3: live measurement UI |
| `AnalysisStep` | `src/components/calibrate/AnalysisStep.tsx` | Step 4: dE chart + gamma + table |
| `UploadStep` | `src/components/calibrate/UploadStep.tsx` | Step 5: upload progress |
| `VerifyStep` | `src/components/calibrate/VerifyStep.tsx` | Step 6: post comparison |
| `LiveGammaChart` | `src/components/calibrate/LiveGammaChart.tsx` | SVG gamma curve (shared) |
| `DeBarChart` | `src/components/calibrate/DeBarChart.tsx` | SVG dE2000 bar chart |
| `PatchDataTable` | `src/components/calibrate/PatchDataTable.tsx` | Color-coded patch results table |
| `YxyReadout` | `src/components/calibrate/YxyReadout.tsx` | Big numeric Yxy display |
| `MeterProfiler` | `src/components/devices/MeterProfiler.tsx` | Standalone profiler wizard (Devices page) |

---

## 7. State Management

Wizard step state is **local to CalibrateView** (React `useState`), not in the global Zustand store. The reasoning:
- Wizard state is ephemeral and should reset on navigation away
- Only `CalibrationService` backend state is shared
- Measurement events update both local wizard state AND global dashboard store (so TopBar shows "Measuring")

```typescript
type WizardStep =
  | "devices"
  | "profiling"
  | "target"
  | "measure"
  | "analyze"
  | "upload"
  | "verify";

interface WizardState {
  step: WizardStep;
  sessionId: string | null;
  config: SessionConfigDto | null;
  measurements: PatchReading[];
  analysis: AnalysisResult | null;
  verification: VerificationResult | null;
}
```

---

## 8. Data Flow

```
User clicks "New Calibration" in DashboardView
  → navigate("/calibrate")
    → CalibrateView mounts (step = "devices")
      → DeviceSelectionStep auto-populates from useDashboardStore
        → User clicks "Next" (optionally checks "Profile meter first")
          → IF profiling: step = "profiling"
            → ProfilingStep starts
              → Backend: start_profiling() → runs MeasurementLoop with profiling patch set
              → Events: profiling-progress → frontend updates LiveGammaChart + PatchDataTable
              → profiling-complete → show correction matrix, user Accepts
          → step = "target"
            → TargetConfigStep shows form
              → User configures targets, clicks "Start Measurement"
                → step = "measure"
                  → MeasurementStep starts
                    → Backend: start_calibration() → GreyscaleAutoCalFlow::run_sync()
                    → Events: calibration-progress → LiveGammaChart updates, YxyReadout updates
                    → patch-complete → PatchDataTable appends row
                  → All patches done → step = "analyze"
                    → AnalysisStep shows DeBarChart + LiveGammaChart + PatchDataTable
                    → User clicks "Apply Corrections"
                      → step = "upload"
                        → Backend uploads LUT + sets white balance
                        → lut-uploaded event
                        → Auto-advance after 2s → step = "verify"
                          → Backend re-measures same patches
                          → Events: verification-complete
                          → VerifyStep shows pre/post dE comparison + pass/fail
                            → User clicks "Save Session" → stores to SQLite
                            → User clicks "Back to Dashboard" → navigate("/")
```

---

## 9. Testing Strategy

### Backend
- Unit test: `CalibrationService::start_calibration` spawns flow correctly
- Unit test: `CalibrationService::abort_calibration` cancels spawned task
- Unit test: Event bridge converts `CalibrationEvent` to Tauri events
- Unit test: Profiling mode with two meters generates correct correction matrix
- Integration test: Full wizard flow with mocks (devices → measure → analyze → upload → verify)

### Frontend
- Component test: `DeviceSelectionStep` renders with mocked connected devices
- Component test: `MeasurementStep` updates chart on mocked events
- Component test: `AnalysisStep` renders dE bar chart from mock data
- Component test: `VerifyStep` shows pass/fail based on pre/post dE
- Component test: `MeterProfiler` standalone flow

---

## 10. Open Questions / Future Work

- **Charting library:** MVP uses SVG for gamma and dE charts. If charts become more complex (3D LUT visualization, CIE diagrams), consider a lightweight library like `recharts` or `d3`.
- **Pause/Resume:** The backend `GreyscaleAutoCalFlow` has a `Paused` state but no explicit resume. The design assumes pause = stop after current patch, resume = restart from next patch. Verify this is acceptable.
- **Session persistence:** "Save Session" stores to SQLite. The exact schema for saving wizard results (not just raw readings) may need a new table.
- **File export:** `.ccmx` export in profiling and session report export are placeholders for Phase 9/10.

---

## 11. Spec Self-Review

- **Placeholder scan:** No TBDs or TODOs.
- **Internal consistency:** Event names and command signatures match the IPC models section. The 6-step flow aligns with backend `GreyscaleAutoCalFlow` pipeline.
- **Scope check:** This is focused enough for a single implementation plan. The standalone profiler is a separate sub-component but shares UI components with the wizard.
- **Ambiguity check:** The profiling step is optional (checkbox in Step 1). If unchecked, flow goes directly from Step 1 → Step 2. If checked, Step 1a is inserted.
