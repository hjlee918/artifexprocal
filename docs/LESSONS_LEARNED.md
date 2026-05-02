# Lessons Learned from v1 — ArtifexProCal Architecture Retrospective

**Date:** 2026-04-30
**Scope:** What went wrong in the first codebase (`archive/v1/`) and what to avoid in the clean-slate rebuild.

---

## 1. Frontend State: Never Hydrated from Backend

**What happened:** `useDashboardStore` (Zustand) only updated via Tauri events. Events only fired when backend commands ran. The store started empty, so every page showed "Disconnected" until something happened — but nothing auto-triggered. `DevicesView` was literally a hardcoded placeholder that never called `get_device_inventory`.

**Impact:** App was unusable on first launch. User had to guess why nothing worked.

**Rule for v2:** Every store that mirrors backend state **must** fetch on mount. Every view must be functional without requiring prior events. Explicit hydration is not optional — it is the default.

---

## 2. CalibrationService Became a 1000-Line God Object

**What happened:** `src-tauri/src/service/state.rs` grew to 824 lines, handling: device connection, session management, calibration flow orchestration, profiling flow, manual flow, event emission, abort handling, storage access, and correction matrix state.

**Impact:**
- Impossible to test in isolation — every test had to set up the full service
- Changing one flow risked breaking another
- No clear separation between IPC commands and domain logic

**Rule for v2:** The Tauri command handler layer (`src-tauri/src/ipc/commands.rs`) must be **thin** — under 100 lines. It delegates to focused engine crates. No business logic in IPC handlers. No `CalibrationService` singleton.

---

## 3. TypeScript Bindings Were Manually Maintained

**What happened:** `src/bindings.ts` was hand-written and quickly drifted from the actual Rust command signatures. Events like `EVENT_CALIBRATION_PROGRESS` were manually maintained constants. When new commands were added, TypeScript types often lagged behind.

**Impact:** `npm run tauri build` failed repeatedly with TypeScript errors. We had to define local interfaces in components (`MeasurementStep.tsx`, `ProfilingStep.tsx`) as workarounds.

**Rule for v2:** `tauri-specta` must auto-generate bindings on every `cargo build`. The build pipeline must fail if bindings are out of sync. Manual `bindings.ts` is forbidden.

---

## 4. Wizard Steps Built Independently, Never Integrated End-to-End

**What happened:** `DeviceSelectionStep`, `TargetConfigStep`, `MeasurementStep`, `AnalysisStep`, etc. were built as isolated components with their own data types (`src/components/calibrate/types.ts`). Each step had its own assumptions about state shape. When we tried to wire them into `CalibrationWizard`, the data contracts didn't match.

**Impact:** The wizard shell (`CalibrationWizard.tsx`) had to do heavy data mapping between steps. State leaked between steps. `CalibrateView` became a complex router that had to know about every step's internal state.

**Rule for v2:** Define the wizard state machine **first**, before any UI. Every step receives the same state object and emits the same action enum. No per-step type definitions. One canonical session state type for the entire wizard.

---

## 5. Three.js Bundle Bloat

**What happened:** `@react-three/fiber` and `@react-three/drei` were added for 3D LUT visualization. The final production bundle was 1.2MB minified, with Three.js contributing the majority. The `LutCubeScene` component was built before any real LUT data could flow to it.

**Impact:** Slow first paint, memory pressure, and the 3D component was ultimately untested with real data.

**Rule for v2:** Do not add Three.js until the calibration flow produces real LUT data that needs visualization. Start with SVG/Canvas for 2D CIE diagrams and gamma curves. Three.js is a Phase 5+ feature, not Phase 1.

---

## 6. Event Channel `blocking_recv` Deadlock Risk

**What happened:** Calibration flows used `calibration_engine::events::EventChannel` with `blocking_recv()` in spawned threads. The receiver thread blocked indefinitely if the sender dropped without sending a termination event.

**Impact:** Threads leaked on error paths. The `cancelled` flag pattern was bolted on to prevent this, but it was fragile and scattered across multiple `useEffect` hooks.

**Rule for v2:** Use async event channels with timeout. Every event loop must have a termination condition (session end, error, abort). No `blocking_recv` in UI-facing code.

---

## 7. Pattern Generator Was Implicitly Connected

**What happened:** `connect_pattern_generator()` was called inside `run_calibration()` if the pattern generator wasn't already connected. This meant the user never saw pattern generator status in the device list. It was invisible until a calibration started.

**Impact:** Users couldn't verify pattern generator connectivity before starting a session. Failures happened mid-flow.

**Rule for v2:** Pattern generator is a first-class device with its own connection UI, inventory entry, and status indicator. No implicit auto-connect.

---

## 8. No Frontend-Backend Integration Testing Until Final Build

**What happened:** We built frontend components with mock data and backend flows with unit tests, but never tested them together until `npm run tauri build` on the final packaging step.

**Impact:** Type mismatches, missing bindings, event name typos, and state sync bugs were only discovered at the last minute.

**Rule for v2:** Every feature must be end-to-end testable before the phase is marked complete. The dev server (`npm run dev`) must exercise the full flow: UI button → Tauri command → Rust logic → event → UI update.

---

## 9. Correction Matrix Application Was Bolted On

**What happened:** Meter profiling generated a correction matrix, but there was no clear path to apply it. We added `active_correction_matrix` to `CalibrationService` and wrapped meters with `CorrectingMeter` in three separate methods (`run_calibration`, `start_manual_calibration`, `measure_manual_patch`).

**Impact:** The matrix application logic was duplicated and easy to miss. AutoCal flow didn't use it at all initially.

**Rule for v2:** Correction matrix is a core concern of the meter HAL layer, not the calibration flow. The `Meter` trait should optionally accept a correction matrix at connection time. The flow never sees it.

---

## 10. Phases Were Too Granular, Leading to Integration Debt

**What happened:** We had 8+ phases with 5–10 tasks each. Each task built a small piece in isolation. By the time we tried to integrate everything in Phase 8, the pieces didn't fit.

**Impact:** Phase 8 became a catch-all "fix everything" phase with no clear scope. The project grew without a working end-to-end flow until the very end.

**Rule for v2:** Fewer phases, each producing a **working, testable, shippable increment**:
- Phase 1: Device discovery + connection UI (works end-to-end)
- Phase 2: Single grayscale AutoCal flow (works end-to-end)
- Phase 3: Analysis + visualization (works end-to-end)
- Phase 4: History, storage, reporting
- Phase 5: 3D LUT, manual mode, profiling

---

## 11. `.docx` Documents Grew Out of Sync with Code

**What happened:** Every design spec was written in both `.md` and `.docx`. The `.docx` versions were never updated after implementation changed. The `.md` versions were sometimes updated, sometimes not.

**Impact:** Multiple conflicting versions of the same spec existed.

**Rule for v2:** `.md` is the single source of truth. `.docx` is generated from `.md` at commit time via a script. Never hand-edit `.docx`.

---

## 12. Mock-Only Testing Hid Real Hardware Behavior

**What happened:** Most integration tests used `hal::mocks::FakeMeter` and `FakeDisplayController`. The real `I1DisplayPro` HID unlock protocol was only tested manually on physical hardware. The real `LgOledController` was never tested against a real TV.

**Impact:** We don't know if the LG LUT upload format is correct. The SDC binary format was never verified.

**Rule for v2:** Every hardware driver must have a "probe" or "self-test" command that can verify connectivity without running a full calibration. Mock tests are fine for flow logic, but hardware tests must be documented and reproducible.

---

## 13. Sync Probe is a Phase 1 Simplification

**What happened:** The `Meter` trait `probe()` method was kept synchronous in Phase 1 because `CalibrationModule::handle_command` is synchronous and no async driver adapters exist yet.

**Impact:** Real drivers (ArgyllCMS PTY, native HID) will require async probe in Phase 2 when `driver_adapter.rs` wraps blocking I/O with `tokio::task::spawn_blocking`.

**Rule for v2:** Revisit `probe` async signature when the first real driver adapter lands in Phase 2. Document the simplification now so the debt is visible, not invisible.

---

## 14. Numeric Grounding: Plans Are Not Sources

**What happened:** During the export milestone review, a "35 columns" claim propagated through two review rounds. The number originated in a plan summary, not in code or spec text. It was only caught when verification asked for source-document confirmation — the spec had no explicit count, and the test had been silently edited to match the remembered number.

**Impact:** A `fix(test):` commit message misframed the audit trail (claimed a 34→35 change that never existed). The fabricated number wasted two review cycles before being grounded.

**Rule for v2:** Any specific number asserted in plans, prompts, or commit messages must be grounded in code or spec text at the time of assertion, not recalled from memory or prior summaries. Applies to both reviewer and implementer.

---

## 15. Pause-and-Surface: Trivial Divergence Is Still Divergence

**What happened:** The schema-shape question (`anyOf` vs `if/then/else`) was surfaced as options before implementing and resolved correctly. But when the column count diverged from a remembered "35," the resolution was silently applied by editing the test without pausing to surface the conflict. The lapse correlated with whether the conflict was recognized as a conflict or misread as an obvious fix.

**Impact:** A commit message misrepresented the change as a correction when it was actually a fabrication cleanup. The real issue — a missing explicit count in the spec — was only caught in a later verification round.

**Rule for v2:** When implementation diverges from spec, always pause and surface the conflict explicitly, even if the resolution looks trivial. The cost of pausing is low; the cost of silently resolving on the wrong side is high.

---

## 16. Trait Expansion Requires Explicit Justification

**What happened:** During the polish IPC sweep, the `Meter` trait was expanded with `probe()` returning `ProbeResult` and `set_config()` methods. Both were named explicitly in the plan, justified against the working pattern "implement only what's explicitly named," and approved before implementation.

**Impact:** The trait expansion shipped cleanly with no downstream breakage and no "oh we also need..." follow-up commits.

**Rule for v2:** Any change to a shared trait must be named explicitly in the plan, justified against working patterns, and approved before implementation. Treat trait expansion as a design decision, not an implementation detail.

---

## 17. Reviewer Prompts: Ask What the Source Says, Not Whether the Claim Is True

**What happened:** A verification prompt asserted "35 columns" as a fact, which closed off the path of "read the spec, find no count, surface that absence." The prompt's framing made the implementer defend a claim rather than investigate a source.

**Impact:** The fabrication was only revealed when the implementer happened to re-read the spec for an unrelated reason. If the prompt had asked "what does the spec say about column count?" the absence would have been surfaced immediately.

**Rule for v2:** Verification prompts should ask "what does source X say?" rather than "is claim Y true?" — the former reveals fabrications that the latter hides.

---

## 18. Schema Is Editable In Place

**What happened:** When the Phase 1 export schema produced misleading representations (empty colorspace with concrete `{0,0,0}` patch values), the resolution was to tighten the schema itself (`anyOf` type unions + `if/then/else` conditional validation) rather than document around the limitation with code comments.

**Impact:** The schema now correctly represents the "no PatternModule" state without workaround documentation. Future phases will extend the schema, not fight it.

**Rule for v2:** While there are no external consumers, prefer fixing the schema over documenting around its limitations. This expires when external consumers exist (Phase 6+).

---

## 19. Datasets With Two Paths Need Both Paths Tested

**What happened:** The CIE 1931 observer data in `color-science` had transcription errors in the z-bar values from 445nm onward (values ~2× too large). The errors were caught because the Planckian locus self-consistency test cross-validated tabulated points against runtime `blackbody_xyz()` computation — two independent paths to the same answer.

**Impact:** Without the cross-validation test, the corrupted data would have propagated into all downstream calculations (blackbody synthesis, CCT, chromaticity diagrams).

**Rule for v2:** Datasets where two independent paths to the answer exist must have both paths tested. Cross-validation catches transcription errors that source citation alone cannot.

---

## 20. Source Attribution at Entry Time, Not Debug Time

**What happened:** Every tabulated dataset in `color-science` (CIE 1931 observer data, Planckian locus table, Sharma DE2000 test cases) includes source citation comments at the moment of entry. This was enforced after the transcription error incident, not as a post-hoc fix.

**Impact:** Future readers can verify data provenance without archaeology. The CIE 1931 data cites `colour-science` v0.4.7 → CIE 15:2018 Table T.1; the DE2000 cases cite Sharma's `ciede2000testdata.txt`.

**Rule for v2:** Every tabulated dataset, formula, or reference value must include a source citation comment at the moment it enters the codebase. Citation is not a cleanup task; it is an entry gate.

---

## 21. Speculative Types Must Be Quarantined

**What happened:** Early in Phase 1, the `hal` crate contained `DisplayController`, `PatternGenerator`, `Lut1D`, `Lut3D`, and other Phase 3+ types. These were moved to a `hal-future-traits` crate clearly marked as speculative, and `hal` was scoped back to ~70 lines exposing only the `Meter` trait.

**Impact:** Active crates stayed focused on Phase 1 concerns. Future types are preserved as design sketches without polluting the active dependency graph.

**Rule for v2:** Types and traits planned for future phases must live in a clearly marked quarantine crate. Promote only when the phase arrives.

---

## 22. Surface Scope Creep at Planning Time

**What happened:** During the cct/duv milestone, the implementer proposed computing cct/duv at export time (deferring to Phase 2). The reviewer caught this as scope creep during the planning phase and directed computation to `MeasurementResult::from_xyz` construction time instead. During the export milestone, speculative `MeasurementResult` fields planned for later phases were similarly caught and deferred.

**Impact:** No speculative fields leaked into the Phase 1 measurement contract. The `MeasurementResult` struct contains only fields that are actually populated by Phase 1 code paths.

**Rule for v2:** When an implementer proposes "X for future use Y" during a milestone for Z, list X explicitly in the plan and explicitly ask whether to defer. Surface scope creep at planning time, not at commit review time.

---

## Summary: The New Rules

| # | Rule |
|---|------|
| 1 | Store hydrates on mount |
| 2 | IPC handlers are thin delegators |
| 3 | Bindings are auto-generated, never hand-written |
| 4 | State machine first, UI second |
| 5 | No Three.js until real data flows |
| 6 | No `blocking_recv` in UI code |
| 7 | Pattern generator is a first-class device |
| 8 | Every phase ships end-to-end |
| 9 | Correction matrix is HAL concern, not flow concern |
| 10 | Fewer phases, larger working increments |
| 11 | `.md` is truth; `.docx` is generated |
| 12 | Hardware drivers need self-test commands |
| 13 | Sync probe is a Phase 1 simplification |
| 14 | Numeric grounding: plans are not sources |
| 15 | Pause-and-surface on any spec divergence |
| 16 | Trait expansion requires explicit justification |
| 17 | Ask what the source says, not whether the claim is true |
| 18 | Schema is editable in place until external consumers exist |
| 19 | Datasets with two paths need both paths tested |
| 20 | Source attribution at entry time, not debug time |
| 21 | Speculative types must be quarantined |
| 22 | Surface scope creep at planning time |
