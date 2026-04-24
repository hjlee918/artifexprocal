# Calibration Software Design — LG OLED & Sony Projector Professional Suite

**Date:** 2026-04-24
**Status:** Draft
**Target:** Professional-grade display calibration, competing with CalMAN Ultimate and ColourSpace INF

---

## 1. Goals & Motivation

Build a unified, cross-platform, lifetime-licensed professional display calibration suite that eliminates the pain points of existing tools:
- **No subscriptions** — one-time purchase, free updates for life
- **Cross-platform** — macOS, Windows, Linux (where CalMAN is Windows-only)
- **Unified workflow** — one tool for meters, pattern generators, display control, LUT generation, and reporting
- **Modern visualization** — GPU-accelerated CIE diagrams, 3D LUT cubes, live measurement graphs
- **Extensible** — plugin architecture for new displays, meters, and pattern generators

---

## 2. Scope (MVP)

### Supported Hardware (V1)
| Category | Devices |
|----------|---------|
| Displays | LG OLED (2018–2025/26, network AutoCal), Sony VPL-VW385ES projector |
| Colorimeters | X-Rite i1 Display Pro Rev.B (2000 nits HDR), X-Rite i1 Pro 2 spectrophotometer |
| Pattern Generators | PGenerator 1.6 (Raspberry Pi 4), LG internal pattern generator (2019+), Ted's LightSpace CMS Calibration Disk templates |

### Supported Targets
- SDR: BT.709, gamma 2.2 / 2.4, D65 white point
- HDR10: BT.2020 container, PQ EOTF (ST.2084), custom tone curves
- Dolby Vision: Config file generation (white/black/RGB data upload)
- Custom: Arbitrary white point, custom gamma, reduced gamut targets

### Core Workflows (V1)
1. **AutoCal Wizard** — automatic calibration of LG OLED via network (greyscale, color gamut, HDR tone curve)
2. **Manual Calibration** — step-by-step SDR/HDR manual adjustment with live meter feedback
3. **3D LUT Generation** — create corrective LUTs for video processors or display upload
4. **Device Profiling** — characterize display native gamut, create meter correction matrices
5. **Validation & Reporting** — pre/post measurement, custom PDF reports, dE analysis

---

## 3. Architecture

### Tech Stack
| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Frontend | React 19 + TypeScript + Tailwind CSS + Three.js/WebGL | Reactive UI, GPU-accelerated scientific visualization |
| Backend | Rust (Tauri) | Memory safety, deterministic hardware I/O, small binaries (~5-15MB), no GC pauses during measurement loops |
| IPC | Tauri Commands + Events | Type-safe, async, bidirectional |
| Database | SQLite (embedded) | Session persistence, calibration history, display profiles |
| Report | HTML/CSS → PDF (via Rust `printpdf` or headless) | Custom layouts, exportable |

### System Diagram
```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (WebView)                        │
│  React + TypeScript + Three.js/WebGL for visualization       │
│  - CIE 1931/1976 diagrams, 3D LUT cubes, tone curves        │
│  - Calibration wizard UI, real-time measurement graphs       │
│  - Report designer with drag-and-drop charts                 │
└────────────────────────┬──────────────────────────────────────┘
                         │ IPC (Tauri commands / events)
┌────────────────────────▼──────────────────────────────────────┐
│                    Backend (Rust)                            │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │ Color       │  │ Hardware     │  │ Calibration Engine  │  │
│  │ Science     │  │ Abstraction  │  │ - Workflow state    │  │
│  │ - XYZ/Lab/  │  │ - Meter I/O  │  │ - Patch sequencing  │  │
│  │   LCh/ICtCp │  │ - Display    │  │ - AutoCal logic     │  │
│  │ - DeltaE    │  │   protocols  │  │ - LUT generation    │  │
│  │ - Gamut     │  │ - Pattern gen│  │ - 3D LUT (33³/65³)  │  │
│  │   conversion│  │   (PGen/Int.)│  │ - 1D LUT (256-1024) │  │
│  └─────────────┘  └──────────────┘  └─────────────────────┘  │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │ Device      │  │ Report       │  │ File I/O            │  │
│  │ Profiling   │  │ Engine       │  │ - ICC profile export│  │
│  │ - Display   │  │ - PDF gen    │  │ - LUT export (.cube│  │
│  │   profile   │  │ - Custom     │  │   .3dl .xml .dat)   │  │
│  │ - Meter     │  │   layouts    │  │ - Session save/load │  │
│  │   correction│  │ - Pre/Post   │  │ - .ccmx profiles    │  │
│  │   matrix    │  │   comparison │  │                     │  │
│  └─────────────┘  └──────────────┘  └─────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## 4. Backend Modules (Rust)

### 4.1 Color Science (`crate:color-science`)
- **Spaces:** XYZ, xyY, Lab, LCh, ICtCp, RGB (linear/gamma), YCbCr
- **Conversions:** Matrix transforms, Bradford adaptation, CAT16
- **DeltaE:** dE 2000 (primary), dE ITU-R BT.2124, dE CMC, dE 1976
- **Gamut:** Triangle intersection, coverage percentage, volume calculation
- **Tone Curves:** Gamma 2.2/2.4, BT.1886, PQ (ST.2084), HLG, custom piecewise

### 4.2 Hardware Abstraction Layer (`crate:hal`)
Trait-based plugin system:

```rust
pub trait Meter: Send + Sync {
    fn connect(&mut self) -> Result<(), MeterError>;
    fn disconnect(&mut self);
    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError>;
    fn set_mode(&mut self, mode: MeterMode) -> Result<(), MeterError>;
    fn model(&self) -> &str;
}

pub trait DisplayController: Send + Sync {
    fn connect(&mut self) -> Result<(), DisplayError>;
    fn set_picture_mode(&mut self, mode: &str) -> Result<(), DisplayError>;
    fn upload_1d_lut(&mut self, lut: &Lut1D) -> Result<(), DisplayError>;
    fn upload_3d_lut(&mut self, lut: &Lut3D) -> Result<(), DisplayError>;
    fn set_white_balance(&mut self, gains: RGBGain) -> Result<(), DisplayError>;
}

pub trait PatternGenerator: Send + Sync {
    fn connect(&mut self) -> Result<(), GenError>;
    fn display_patch(&mut self, color: &RGB) -> Result<(), GenError>;
    fn display_pattern(&mut self, pattern: &Pattern) -> Result<(), GenError>;
}
```

**V1 Implementations:**
- `meter-xrite`: i1 Display Pro (via HID), i1 Pro 2 (via USB/Argyll driver bridge)
- `display-lg-oled`: HTTP REST API for AutoCal (2018–2026 models)
- `display-sony-projector`: RS-232 / IP control for VPL-VW385ES
- `pattern-pgenerator`: HTTP API for PGenerator 1.6 on Raspberry Pi
- `pattern-lg-internal`: LG 2019+ built-in pattern generator via network

### 4.3 Calibration Engine (`crate:calibration`)
- **Session Manager:** SQLite-backed, pause/resume, rollback to any step
- **Patch Sequencer:** Sequential, random, saturation sweeps, greyscale ramps, custom CSV templates
- **Measurement Loop:** Dark reading subtraction, repeated samples (N=3–10) for noise reduction, auto-ranging, outlier rejection
- **AutoCal Logic:** Iterative greyscale balance, CMS saturation adjustment, HDR tone curve fitting
- **1D LUT Generator:** Per-channel curve fitting (spline, polynomial, lookup). Supports greyscale-only (RGB combined) or per-channel independent.
- **3D LUT Generator:** Volumetric interpolation (tetrahedral, trilinear). Supports 17³, 33³, 65³ sizes. Optimized for OLED near-black handling (non-linear spacing).

### 4.4 Device Profiling (`crate:profiling`)
- **Display Characterization:** Measures native RGB primaries, white/black points, gamma response, peak luminance. Generates `.ccss` style spectral data or matrix-based profile.
- **Meter Profiling:** Measures a reference target set with both i1 Pro 2 (spectral) and i1 Display Pro (colorimetric). Generates correction matrix (`.ccmx`) or spectral response data.
- **Profile Storage:** SQLite + JSON export for sharing/display databases.

### 4.5 Report Engine (`crate:reporting`)
- **Data Capture:** Every measurement step automatically saved pre/post with timestamps
- **Templates:** Built-in templates (quick report, detailed, Netflix validation)
- **Custom Designer:** Drag-and-drop layout with charts, tables, images (inspired by CalMAN Design Mode)
- **Export:** PDF, HTML, CSV raw data

---

## 5. Frontend Modules (React/TypeScript)

### 5.1 Visualization Engine
- **CIE Diagram (`<CIEDiagram />`):** 1931 xy or 1976 u'v'. Gamut triangles, measured points, target overlays, dE vectors. Three.js for GPU rendering.
- **3D LUT Cube (`<LUTCubeViewer />`):** Rotatable cube showing correction vectors, error heatmap. D3.js or raw Three.js.
- **Grayscale Tracker (`<GrayscalePlot />`):** RGB balance bars, gamma curve overlay, CCT tracking, dE per step. Recharts or Victory.
- **ColorChecker (`<ColorCheckerGrid />`):** 24/140 patch grid with dE values, skin tone emphasis.
- **Color Volume (`<VolumeChart />`):** 3D scatter or coverage percentage bars.
- **Real-time Monitor:** Live XYZ/xy readings, stability meter, integration progress.

### 5.2 Calibration Wizards
- **AutoCal Wizard:** Step-by-step for LG OLED. Display selection → meter setup → pattern generator choice → pre-measurement → calibration → post-measurement → report.
- **Manual Calibration:** Live feedback for manual TV adjustment. User changes setting on TV → software shows before/after meter reading instantly.
- **3D LUT Wizard:** Source profile → target space → patch generation → measurement → LUT export.
- **Profiling Wizard:** Meter selection (reference + field) → patch set → measurement → correction matrix export.

### 5.3 Dashboard
- **Session History:** All past calibrations, searchable, filterable
- **Device Inventory:** Known displays, meters, saved profiles
- **Quick Actions:** One-click re-run last calibration, validation check

---

## 6. Data Flow

### Calibration Session (AutoCal)
```
1. User selects Display (LG OLED) + Meter (i1 Display Pro) + Pattern Gen (PGenerator)
2. Backend: connect_all() → verify communication
3. Frontend: show pre-measurement wizard
4. Backend: run_patch_sequence(greyscale_21pt) → store in SQLite
5. Backend: analyze_greyscale() → compute corrections
6. Backend: upload_to_display(LG, corrections) via AutoCal API
7. Backend: run_patch_sequence(color_saturation) → store
8. Backend: analyze_gamut() → compute CMS corrections
9. Backend: upload_to_display(LG, cms_corrections)
10. Backend: run_post_measurement(full_verification)
11. Frontend: render pre/post comparison charts
12. Backend: generate_pdf_report(session_id)
```

### 3D LUT Generation
```
1. User selects Source Profile (measured native gamut) + Target Space (BT.2020/PQ)
2. Backend: generate_patch_set(cube_size=33³) → send to pattern generator
3. Backend: measure_all_patches() → store XYZ/Lab for each RGB triplet
4. Backend: build_3d_lut(measured, target, interpolation=tetrahedral)
5. Backend: preview_lut_quality() → compute max/average dE
6. Frontend: show LUT cube visualization + error heatmap
7. Backend: export_lut(format=.cube/.3dl/.xml)
```

---

## 7. File Formats & Interop

| Format | Import | Export | Purpose |
|--------|--------|--------|---------|
| `.cube` | Yes | Yes | DaVinci Resolve, madVR, general 3D LUT |
| `.3dl` | Yes | Yes | Lustre, Autodesk |
| `.xml` | Yes | Yes | Dolby Vision config, ColorBox |
| `.dat` | Yes | Yes | madVR 3D LUT |
| `.icc` / `.icm` | Yes | Yes | ICC v4 profiles |
| `.ccmx` | Yes | Yes | Argyll meter correction matrix |
| `.ccss` | Yes | Yes | Argyll spectral sample data |
| `.csv` | Yes | Yes | Raw measurement data exchange |
| `.json` | Yes | Yes | Session save, display database |

---

## 8. Testing Strategy

- **Unit Tests:** Color science conversions (property-based with `quickcheck`), DeltaE accuracy against reference datasets
- **Integration Tests:** Mock HAL implementations (fake meter that returns known XYZ values), verify entire calibration pipeline end-to-end
- **Hardware Tests:** Real device testing with your i1 Display Pro + LG OLED (manual QA, not CI)
- **Visual Regression:** Screenshot comparison for CIE diagrams, LUT cubes

---

## 9. Project Structure

```
calibration-suite/
├── src-tauri/               # Rust backend
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── color_science/   # XYZ, Lab, DeltaE, gamut math
│   │   ├── hal/             # Hardware abstraction traits
│   │   ├── hal_meters/      # i1 Display Pro, i1 Pro 2
│   │   ├── hal_displays/    # LG OLED, Sony projector
│   │   ├── hal_patterns/    # PGenerator, LG internal
│   │   ├── calibration/     # Engine, session manager
│   │   ├── profiling/       # Display & meter profiling
│   │   ├── reporting/       # PDF/HTML generation
│   │   └── ipc/             # Tauri command handlers
│   └── tauri.conf.json
├── src/                     # React frontend
│   ├── components/
│   │   ├── visualization/   # CIEDiagram, LUTCube, etc.
│   │   ├── wizards/         # AutoCal, Manual, LUT, Profiling
│   │   └── layout/          # Sidebar, header, dashboard
│   ├── hooks/               # useCalibration, useMeter, etc.
│   ├── stores/              # Zustand state management
│   └── types/               # Shared TypeScript types
├── docs/
├── tests/
└── package.json
```

---

## 10. Future Expansion (Post-MVP)

- **Additional Displays:** Samsung QLED (2019–2026), Panasonic OLED, Flanders Scientific, EIZO ColorEdge
- **Additional Meters:** Klein K-10, Jeti Specbos, Photo Research PR-series, Portrait C6 HDR2000
- **Additional Pattern Generators:** VideoForge Pro, Murideo Six-G, DaVinci Resolve Pattern Generator
- **Video Wall Matching:** Multi-panel calibration for Planar/Barco walls
- **Netflix Validation Workflows:** Specific pre/post checks for post-production mastering
- **Network Licensing:** Optional floating licenses for facility deployments
- **Cloud Sync:** Calibration history sync across workstations

---

## 11. Open Questions / Decisions

1. **Licensing:** Tiered — Lite / Pro / Ultimate (like CalMAN's model but with lifetime purchase, not subscription)
2. **Offline-first:** Entirely offline, no account validation required
3. **Open Source Core:** Color science backend and HAL traits will be open-source; UI and hardware driver implementations may be proprietary
4. **Measurement Speed vs. Accuracy:** Fast default mode (Lightning LUT style) with optional high-precision iterative refinement

---

## 12. References

- [CalMAN Ultimate Product Page](https://store.portrait.com/catalog/product/view/id/159/s/calman-ultimate/)
- [CalMAN Home for LG](https://store.portrait.com/calman-home-for-lg.html)
- [ColourSpace INF Info](https://www.lightillusion.com/colourspace.html)
- [ColourSpace Why ColourSpace?](https://lightillusion.com/why_colourspace.html)
- [TFT Central CalMAN Ultimate Review](https://www.tftcentral.co.uk/articles/calman_ultimate.htm)
