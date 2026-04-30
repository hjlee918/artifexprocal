# Phase 4c — Visualization Engine Design

> **Date:** 2026-04-26
> **Status:** Approved

**Goal:** Add scientific color-accuracy visualizations to the calibration workflow: CIE chromaticity diagrams, enhanced grayscale tracking, ColorChecker validation grids, and a Three.js scaffold for future 3D LUT rendering.

**Architecture:** Mixed rendering approach — Canvas 2D for CIE, SVG for grayscale, HTML/CSS for ColorChecker, Three.js scaffold for 3D. All driven from existing `CalibrationEvent::ReadingsComplete` XYZ data; no backend state machine changes.

**Tech Stack:** React 19 + TypeScript, HTML5 Canvas 2D, SVG, Three.js + @react-three/fiber, Tailwind CSS

---

## 1. Rendering Strategy

| Visualization | Technology | Rationale |
|--------------|-----------|-----------|
| CIE 1931/1976 Diagram | Canvas 2D | Lightweight, pixel-perfect 2D primitives, efficient for thousands of spectral locus points |
| Grayscale Tracker | SVG | Extends existing `LiveGammaChart`/`DeBarChart`; native axes, labels, interactivity |
| ColorChecker Grid | HTML/CSS | Simplest, most accessible, responsive without extra library |
| 3D LUT Cube | Three.js + @react-three/fiber | GPU-accelerated, reusable `<ThreeCanvas>` wrapper; actual cube deferred until 3D LUT generation |

---

## 2. Component Inventory

### 2.1 CIE Diagram (`src/components/visualizations/CIEDiagram.tsx`)
- Renders spectral locus (precomputed xy coordinates from backend)
- Draws target gamut triangle (from `SessionConfigDto.target_space`)
- Draws measured gamut triangle (from patch readings)
- White point marker (target vs measured)
- Optional: dE vectors from measured to target primaries
- Toggle between CIE 1931 xy and CIE 1976 u'v'

### 2.2 Grayscale Tracker (`src/components/visualizations/GrayscaleTracker.tsx`)
- Replaces/enhances `LiveGammaChart` + `DeBarChart` in `AnalysisStep`
- RGB balance bars per patch level
- Gamma curve overlay (target dashed, measured solid)
- CCT (correlated color temperature) tracking per step
- dE2000 per step bar chart
- White point drift (x,y) scatter overlay

### 2.3 ColorChecker Grid (`src/components/visualizations/ColorCheckerGrid.tsx`)
- 24-patch or 140-patch grid layout
- Each patch shows measured vs target color
- dE2000 value overlaid on each patch
- Skin tone emphasis (patches 19-24 in classic 24-patch)
- Pass/fail color coding (green < 1, yellow < 3, red >= 3)

### 2.4 Three.js Scaffold
- `src/components/visualizations/ThreeCanvas.tsx` — reusable R3F `<Canvas>` wrapper with shared lighting, camera defaults, orbit controls
- `src/components/visualizations/LutCubeScene.tsx` — placeholder scene with a simple wireframe cube; wired to a future data prop

### 2.5 Shared Utilities
- `src/lib/colorMath.ts` — XYZ→xy, XYZ→uv (for CIE 1976), CCT approximation, target gamut computation from color space name
- `src/lib/spectralLocus.ts` — type definitions for spectral locus data; actual data loaded from backend

---

## 3. Backend Additions (Minimal)

Two new Tauri commands in `src-tauri/src/ipc/commands.rs`:

1. **`get_spectral_locus(diagram: String)`** → `Vec<[f64; 2]>`
   - Returns precomputed CIE 1931 xy or CIE 1976 u'v' spectral locus coordinates
   - Data stored as a `const` array in a new `crates/color-science/src/diagrams.rs`
   - ~400 points, trivial bandwidth; no need for caching

2. **`get_target_gamut(target_space: String)`** → `GamutDto`
   - Returns red, green, blue primary xy coordinates + white point xy for the named target space
   - Supports: "Rec.709", "Rec.2020", "DCI-P3", "sRGB", "Adobe RGB"
   - Data from known standards matrices; no measurement needed

New IPC models:
```typescript
interface GamutDto {
  red: [number, number];
  green: [number, number];
  blue: [number, number];
  white: [number, number];
}
```

---

## 4. Data Flow

1. Backend emits `CalibrationEvent::ReadingsComplete { patch_index, xyz, rgb, std_dev, stable }`
2. Frontend `MeasurementStep` stores readings in `useDashboardStore`
3. `AnalysisStep` passes readings to:
   - `GrayscaleTracker` — maps each reading to gamma/dE/RGB balance/CCT
   - `CIEDiagram` — extracts primaries (max R, max G, max B patches) + white (100% white patch) to draw triangles
4. `VerifyStep` (future) passes post-cal readings to `ColorCheckerGrid`

No backend workflow changes. Frontend-only computation.

---

## 5. Integration Points

### AnalysisStep (`src/components/calibrate/AnalysisStep.tsx`)
Replace the existing chart section:
```
Before: LiveGammaChart + DeBarChart (two separate SVGs)
After:  GrayscaleTracker (unified SVG) + CIEDiagram (Canvas)
```

### New Route (optional)
Add a "Visualize" nav item that shows all charts with mock/test data for exploration. Not required for wizard flow.

---

## 6. Testing Strategy

- **Unit:** `colorMath.ts` functions with known values (D65 xy = 0.3127, 0.3290)
- **Component:** Mount `CIEDiagram` with mock gamut data, assert canvas path commands
- **Component:** Mount `GrayscaleTracker` with mock readings, assert SVG rect count and path data
- **Component:** Mount `ColorCheckerGrid` with 24 patches, assert 24 cells rendered
- **Integration:** Full `AnalysisStep` with mock store, verify all sub-components render

---

## 7. Scope Decomposition

This phase is a single implementation plan — all visualizations share the same data source (`readings: PatchReading[]`) and color math utilities. The 3D LUT scaffold is intentionally minimal (just the wrapper component).

---

## 8. Open Questions (Resolved)

| Question | Decision |
|----------|----------|
| Which visualization first? | Grayscale Tracker (B), but build all 4 in this phase |
| Rendering technology? | Mixed: Canvas 2D, SVG, HTML/CSS, Three.js |
| Scope approach? | Hybrid — build 3 now + scaffold 3D |
| Backend changes? | Minimal — 2 new commands for reference data |
