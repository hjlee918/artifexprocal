# 3D LUT Generation Design

## Goal

Implement a professional-grade 3D LUT generation system with tetrahedral interpolation, tiered calibration modes (Grayscale / Grayscale+3D / Full 3D), optimized subset patch measurement, 17³ and 33³ output sizes, and both direct LG OLED upload plus `.cube`/`.3dl`/`.xml` file export.

## Architecture

The system follows a **hybrid pure-engine + side-effect-adapter** architecture. `Lut3DEngine` is 100% pure (no I/O, no async, no HAL calls). It accepts a `PatchSet` and produces a `Lut3D`. Side-effect adapters handle I/O: measurement, upload to LG OLED, and file export. This mirrors the existing `GreyscaleAutoCalFlow` pattern and maximizes testability.

## Tech Stack

- **Rust:** `kd-tree` or custom k-d tree for nearest-neighbor lookup, `nalgebra` for vector math, existing `color_science` crate for XYZ/RGB conversions
- **Frontend:** React + Three.js / React Three Fiber for 3D LUT cube visualization, existing `AnalysisStep` panel extended with 3D LUT tab

---

## Data Structures

### CalibrationTier

Drives patch count and computation path.

```rust
pub enum CalibrationTier {
    GrayscaleOnly,      // 21–33 grayscale patches, skip 3D LUT
    GrayscalePlus3D,    // grayscale + ~150–250 optimized 3D subset
    Full3D,             // grayscale + ~500–1000 full optimized subset
}
```

### PatchSet

Container for all target/displayed colors.

```rust
pub struct PatchSet {
    pub patches: Vec<Patch>,
    pub strategy: PatchStrategy,
}

pub struct Patch {
    pub target_rgb: [u16; 3],      // 10-bit (0–1023), what we asked for
    pub measured_xyz: Option<XYZ>, // populated after measurement
    pub patch_index: usize,
}

pub enum PatchStrategy {
    Grayscale(usize),       // e.g. Grayscale(21)
    OptimizedSubset(usize), // N patches sampled from 3D grid + near-neutral emphasis
}
```

### Lut3D

Extends the existing `hal` type with metadata.

```rust
pub struct Lut3D {
    pub data: Vec<RGB>,       // length = size³
    pub size: usize,          // 17 or 33
    pub input_bit_depth: u8,  // 10 for LG iTPG
    pub output_bit_depth: u8, // 10 or 12
}
```

### TetrahedralInterpolator

Core lookup engine.

```rust
pub struct TetrahedralInterpolator {
    pub lut: Lut3D,
}

impl TetrahedralInterpolator {
    pub fn lookup(&self, r: f64, g: f64, b: f64) -> RGB { /* tetrahedral interpolation */ }
}
```

---

## Patch Generation Strategy (Optimized Subset)

For `GrayscalePlus3D` and `Full3D` tiers, the patch generator produces fewer patches than a naive grid (17³ = 4913 or 33³ = 35937) while capturing enough data for smooth tetrahedral interpolation.

1. **Grayscale ramp** — always included (21 patches for standard, 33 for high-precision), spaced non-linearly to capture gamma behavior
2. **Primary axes** — R, G, B ramps from 0→max at ~5 intermediate steps each
3. **Near-neutral emphasis** — dense sampling in the central 20% of RGB cube where skin tones and gray live; this is where dE is most perceptible
4. **Boundary samples** — corners and edges of the cube to anchor the interpolation
5. **Random jitter** — slight perturbation to non-critical samples to reduce structured error

**Patch counts:**
- `GrayscaleOnly`: 21 patches
- `GrayscalePlus3D`: 21 grayscale + ~180 3D subset = ~200 total
- `Full3D`: 33 grayscale + ~600 3D subset = ~630 total

Patches are sorted by `patch_index` with grayscale first for sequential display on iTPG.

---

## Tetrahedral Interpolation

Tetrahedral interpolation splits each RGB cube voxel into 6 tetrahedra.

For a query point `(r, g, b)` normalized to `[0, 1]`:

1. Find enclosing cube corner `(r0, g0, b0)` where `r0 = floor(r * (size-1))`, etc.
2. Compute fractional offsets `dr = r - r0`, `dg = g - g0`, `db = b - b0`
3. Sort `dr, dg, db` to determine which of the 6 tetrahedra the point falls into
4. Barycentric interpolation using the 4 vertices of that tetrahedron

**Sparse→Dense Pipeline:**
- Build a sparse 3D mapping from target RGB → measured XYZ
- For each grid point, find nearest measured neighbors via k-d tree
- Interpolate in XYZ space (not RGB) using tetrahedral weights
- Convert interpolated XYZ → target RGB (inverse of display transform)

Result: a dense `Lut3D` where every grid point maps target RGB → corrected RGB.

---

## LUT Computation Pipeline

```
Measured PatchSet
       │
       ▼
┌──────────────┐
│ Sparse Grid  │  Map measured patches into 3D RGB space (k-d tree)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Tetrahedral  │  Interpolate unmeasured points in XYZ
│ Interpolator │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ XYZ→RGB      │  Convert back to display RGB (inverse transform)
│ (inverse)    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Downsample   │  33³ → 17³ if needed for Alpha 7 chip
│ (average)    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Lut3DOutput  │  Ready for upload or file export
└──────────────┘
```

The pipeline is deterministic and pure — given the same `PatchSet`, it produces the same `Lut3D`. This makes it trivially unit-testable with synthetic data.

---

## Output Adapters

All output adapters implement:

```rust
pub trait Lut3DOutput {
    fn output(&self, lut: &Lut3D, target_space: &str) -> Result<(), OutputError>;
}
```

### Lut3DUploadAdapter

Talks to LG OLED calibration API:
- Converts `Lut3D` to TV-expected format (10-bit RGB triplets, size-dependent)
- Calls `upload_3d_lut_bt709_from_file()` or `upload_3d_lut_bt2020_from_file()` via existing WebSocket HAL
- Handles Alpha 7 (17³) vs Alpha 9 Gen 4 (33³) chip detection from `get_software_info`

### Lut3DFileExportAdapter

Writes industry-standard formats:
- `.cube` — DaVinci Resolve / Photoshop format (ASCII float triplets, `LUT_3D_SIZE` header)
- `.3dl` — Autodesk format (integer 10-bit or 12-bit values)
- `.xml` — Dolby Vision config (MVP may skip if metadata schema is not finalized)

---

## Integration with AutoCal Flow

The existing `GreyscaleAutoCalFlow` gains a `CalibrationTier` parameter.

In `Analyzing` state:

```rust
match tier {
    GrayscaleOnly => compute_1d_lut(),
    GrayscalePlus3D | Full3D => {
        compute_1d_lut(); // still needed for tone curve
        compute_3d_lut();  // additional volumetric correction
    }
}
```

The frontend `CalibrateView` wizard already has a "profiling" step placeholder. It will be wired to select the tier before measurement begins.

---

## Frontend Visualization

For 3D LUT, the `AnalysisStep` gains a new tab/panel:

- **3D LUT Cube** — existing `LutCubeScene` in `ThreeCanvas`, colored by correction magnitude (white = no correction, red = large correction)
- **Correction Heatmap** — 2D slice view showing correction vectors per color region
- **dE Improvement Summary** — before/after table comparing grayscale-only vs 3D-corrected dE on a reference patch set

---

## Testing Strategy

1. **Unit tests for `Lut3DEngine`**
   - Synthetic patch set → known LUT output (identity transform for perfect display)
   - Tetrahedral interpolation matches exact corner values
   - Downsample 33³ → 17³ preserves averages within 0.1%

2. **Integration tests for `Lut3DAutoCalFlow`**
   - Mock meter returning predetermined XYZ
   - Verify state machine transitions through all tiers

3. **File format tests**
   - Export `.cube` and `.3dl`, parse back, assert round-trip equality

4. **Frontend tests**
   - `LutCubeScene` renders without WebGL errors
   - `AnalysisStep` switches between grayscale and 3D tabs

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architecture | Hybrid (pure engine + adapters) | Matches existing flow, testable, extensible |
| Interpolation | Tetrahedral | Industry standard, smooth neutrals |
| LUT sizes | 17³ and 33³ | Covers Alpha 7 and Alpha 9 Gen 4 chips |
| Patch strategy | Optimized subset | ~200–630 patches vs 35K naive, keeps sessions fast |
| Output | Direct upload + file export | AutoCal workflow + post-production flexibility |
| Integration | Extend `GreyscaleAutoCalFlow` | Reuse existing measurement/upload infrastructure |
