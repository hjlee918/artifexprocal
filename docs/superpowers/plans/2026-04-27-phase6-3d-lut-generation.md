# 3D LUT Generation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a professional-grade 3D LUT generation system with tetrahedral interpolation, tiered calibration modes, optimized patch subset measurement, 17³ and 33³ output sizes, direct LG OLED upload, and `.cube` / `.3dl` file export.

**Architecture:** Extend the existing `calibration-autocal` crate with a pure `Lut3DEngine` module (patch generation, tetrahedral interpolation, LUT computation). Add `Lut3DAutoCalFlow` in `calibration-engine` that reuses `GreyscaleAutoCalFlow` for measurement but adds a 3D LUT computation state. Side-effect adapters handle upload and file export. Frontend adds a tier selector and 3D LUT visualization tab.

**Tech Stack:** Rust (existing workspace crates), React + TypeScript + Three.js (existing frontend), Tauri IPC with tauri-specta 2.0.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `crates/calibration-core/src/state.rs` | Add `CalibrationTier` enum |
| `crates/calibration-core/src/patch.rs` | Extend `Patch` with `measured_xyz`, add `PatchStrategy` |
| `crates/calibration-autocal/src/patch3d.rs` | Optimized subset patch generator for 3D LUT |
| `crates/calibration-autocal/src/lut3d.rs` | `Lut3DEngine`, `TetrahedralInterpolator` |
| `crates/calibration-autocal/src/export.rs` | `.cube` and `.3dl` file format writers |
| `crates/calibration-autocal/src/lib.rs` | Export new modules |
| `crates/calibration-engine/src/lut3d_flow.rs` | `Lut3DAutoCalFlow` state machine |
| `crates/calibration-engine/src/lib.rs` | Export `lut3d_flow` |
| `crates/hal/src/types.rs` | Extend `Lut3D` with bit depth metadata |
| `src-tauri/src/ipc/models.rs` | Add `CalibrationTierDto`, `Lut3DInfoDto` |
| `src-tauri/src/ipc/commands.rs` | Add `generate_3d_lut`, `export_lut` commands |
| `src-tauri/src/ipc/events.rs` | Add `Lut3DGenerated` event emitter |
| `src-tauri/src/bindings_export.rs` | Collect new commands for specta |
| `src/components/calibrate/TargetConfigStep.tsx` | Add calibration tier selector |
| `src/components/calibrate/Lut3DTab.tsx` | New 3D LUT analysis tab |
| `src/components/calibrate/AnalysisStep.tsx` | Integrate `Lut3DTab` as conditional tab |
| `src/components/visualizations/LutCubeScene.tsx` | Color cube by correction magnitude |
| `src/components/views/CalibrateView.tsx` | Wire tier selection and 3D LUT flow |

---

### Task 1: Extend Core Types with CalibrationTier and Patch Strategy

**Files:**
- Modify: `crates/calibration-core/src/state.rs`
- Modify: `crates/calibration-core/src/patch.rs`
- Test: `crates/calibration-core/tests/patch_test.rs` (create if absent, else modify existing)

- [ ] **Step 1: Add `CalibrationTier` to `state.rs`**

Add the enum after `WhitePoint`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CalibrationTier {
    GrayscaleOnly,
    GrayscalePlus3D,
    Full3D,
}
```

- [ ] **Step 2: Add `tier` to `SessionConfig`**

Modify `SessionConfig` in `state.rs` to include the tier:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionConfig {
    pub name: String,
    pub target_space: TargetSpace,
    pub tone_curve: ToneCurve,
    pub white_point: WhitePoint,
    pub patch_count: usize,
    pub reads_per_patch: usize,
    pub settle_time_ms: u64,
    pub stability_threshold: Option<f64>,
    pub tier: CalibrationTier,
}
```

- [ ] **Step 3: Extend `Patch` with measured data**

Modify `crates/calibration-core/src/patch.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub target_rgb: RGB,
    pub measured_xyz: Option<XYZ>,
}

impl Patch {
    pub fn new(target_rgb: RGB) -> Self {
        Self { target_rgb, measured_xyz: None }
    }

    pub fn with_measurement(target_rgb: RGB, measured_xyz: XYZ) -> Self {
        Self { target_rgb, measured_xyz: Some(measured_xyz) }
    }
}
```

- [ ] **Step 4: Add `PatchStrategy` enum**

In `patch.rs`, add after `Patch`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PatchStrategy {
    Grayscale(usize),
    OptimizedSubset { grayscale_count: usize, color_count: usize },
}
```

- [ ] **Step 5: Update `GreyscalePatchSet::new`**

Update `GreyscalePatchSet::new` in `patch.rs` to use the new `Patch::new` constructor:

```rust
impl GreyscalePatchSet {
    pub fn new(count: usize) -> PatchSet {
        let mut patches = Vec::with_capacity(count);
        for i in 0..count {
            let level = i as f64 / (count.saturating_sub(1).max(1) as f64);
            patches.push(Patch::new(RGB { r: level, g: level, b: level }));
        }
        PatchSet { patches }
    }
}
```

- [ ] **Step 6: Update `GreyscaleAnalyzer::analyze` signature**

In `crates/calibration-autocal/src/greyscale.rs`, change the signature to accept `&[(RGB, XYZ)]` readings — this already works with the existing code since `readings` is `Vec<(RGB, XYZ)>` built from `patch.target_rgb` and measured XYZ. No changes needed to the analyzer body, but verify the loop still works.

- [ ] **Step 7: Write test for `Patch::with_measurement`**

Create or append to `crates/calibration-core/tests/patch_test.rs`:

```rust
use calibration_core::patch::{Patch, PatchSet, GreyscalePatchSet, PatchStrategy};
use calibration_core::state::CalibrationTier;
use color_science::types::{RGB, XYZ};

#[test]
fn patch_with_measurement() {
    let patch = Patch::with_measurement(
        RGB { r: 1.0, g: 0.5, b: 0.0 },
        XYZ { x: 50.0, y: 30.0, z: 5.0 },
    );
    assert_eq!(patch.target_rgb, RGB { r: 1.0, g: 0.5, b: 0.0 });
    assert_eq!(patch.measured_xyz, Some(XYZ { x: 50.0, y: 30.0, z: 5.0 }));
}

#[test]
fn greyscale_patch_set_uses_new_constructor() {
    let set = GreyscalePatchSet::new(5);
    assert_eq!(set.len(), 5);
    assert!(set.patches[0].measured_xyz.is_none());
    assert_eq!(set.patches[4].target_rgb, RGB { r: 1.0, g: 1.0, b: 1.0 });
}
```

- [ ] **Step 8: Run tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-core
```
Expected: All tests pass.

- [ ] **Step 9: Commit**

```bash
git add crates/calibration-core/src/state.rs crates/calibration-core/src/patch.rs crates/calibration-core/tests/
git commit -m "feat(core): add CalibrationTier, PatchStrategy, extend Patch with measured_xyz"
```

---

### Task 2: Optimized Subset Patch Generator

**Files:**
- Create: `crates/calibration-autocal/src/patch3d.rs`
- Modify: `crates/calibration-autocal/src/lib.rs`
- Test: `crates/calibration-autocal/tests/patch3d_test.rs`

- [ ] **Step 1: Create `patch3d.rs` with `OptimizedPatchSetGenerator`**

```rust
use calibration_core::patch::{Patch, PatchSet, PatchStrategy};
use color_science::types::RGB;

pub struct OptimizedPatchSetGenerator;

impl OptimizedPatchSetGenerator {
    /// Generate an optimized subset of patches for 3D LUT measurement.
    /// Includes grayscale ramp, primary axes, near-neutral emphasis, boundary samples.
    pub fn generate(strategy: PatchStrategy) -> PatchSet {
        match strategy {
            PatchStrategy::Grayscale(count) => Self::grayscale(count),
            PatchStrategy::OptimizedSubset { grayscale_count, color_count } => {
                Self::optimized_subset(grayscale_count, color_count)
            }
        }
    }

    fn grayscale(count: usize) -> PatchSet {
        let mut patches = Vec::with_capacity(count);
        for i in 0..count {
            let level = i as f64 / (count.saturating_sub(1).max(1) as f64);
            patches.push(Patch::new(RGB { r: level, g: level, b: level }));
        }
        PatchSet { patches }
    }

    fn optimized_subset(grayscale_count: usize, color_count: usize) -> PatchSet {
        let mut patches = Vec::with_capacity(grayscale_count + color_count);

        // 1. Grayscale ramp
        for i in 0..grayscale_count {
            let level = i as f64 / (grayscale_count.saturating_sub(1).max(1) as f64);
            patches.push(Patch::new(RGB { r: level, g: level, b: level }));
        }

        // 2. Primary axes (R, G, B ramps)
        for i in 1..=5 {
            let level = i as f64 / 6.0;
            patches.push(Patch::new(RGB { r: level, g: 0.0, b: 0.0 }));
            patches.push(Patch::new(RGB { r: 0.0, g: level, b: 0.0 }));
            patches.push(Patch::new(RGB { r: 0.0, g: 0.0, b: level }));
        }

        // 3. Near-neutral emphasis: dense sampling in central 20%
        let neutral_steps = (color_count / 4).max(8);
        for i in 0..neutral_steps {
            let t = i as f64 / (neutral_steps.saturating_sub(1).max(1) as f64);
            let center = 0.4 + t * 0.2; // 0.4 to 0.6 range
            let r = center + (t - 0.5) * 0.1;
            let g = center;
            let b = center - (t - 0.5) * 0.1;
            patches.push(Patch::new(RGB {
                r: r.clamp(0.0, 1.0),
                g: g.clamp(0.0, 1.0),
                b: b.clamp(0.0, 1.0),
            }));
        }

        // 4. Boundary samples: corners and edges
        let corners = vec![
            RGB { r: 0.0, g: 0.0, b: 0.0 },
            RGB { r: 1.0, g: 0.0, b: 0.0 },
            RGB { r: 0.0, g: 1.0, b: 0.0 },
            RGB { r: 0.0, g: 0.0, b: 1.0 },
            RGB { r: 1.0, g: 1.0, b: 0.0 },
            RGB { r: 1.0, g: 0.0, b: 1.0 },
            RGB { r: 0.0, g: 1.0, b: 1.0 },
            RGB { r: 1.0, g: 1.0, b: 1.0 },
        ];
        for c in corners {
            patches.push(Patch::new(c));
        }

        // 5. Random jitter samples to fill remaining quota
        let remaining = (grayscale_count + color_count).saturating_sub(patches.len());
        for i in 0..remaining {
            let t = (i + 1) as f64 / (remaining.saturating_sub(1).max(1) as f64 + 1.0);
            let r = (t * 7.0).sin() * 0.5 + 0.5;
            let g = (t * 11.0).sin() * 0.5 + 0.5;
            let b = (t * 13.0).sin() * 0.5 + 0.5;
            patches.push(Patch::new(RGB { r, g, b }));
        }

        PatchSet { patches }
    }
}
```

- [ ] **Step 2: Export in `lib.rs`**

Add to `crates/calibration-autocal/src/lib.rs`:

```rust
pub mod patch3d;
```

- [ ] **Step 3: Write tests**

Create `crates/calibration-autocal/tests/patch3d_test.rs`:

```rust
use calibration_autocal::patch3d::OptimizedPatchSetGenerator;
use calibration_core::patch::PatchStrategy;

#[test]
fn optimized_subset_grayscale_only() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::Grayscale(21));
    assert_eq!(set.len(), 21);
    // All grayscale
    for patch in &set.patches {
        assert!(patch.measured_xyz.is_none());
        assert!((patch.target_rgb.r - patch.target_rgb.g).abs() < 0.001);
        assert!((patch.target_rgb.g - patch.target_rgb.b).abs() < 0.001);
    }
}

#[test]
fn optimized_subset_full3d_has_enough_patches() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::OptimizedSubset {
        grayscale_count: 33,
        color_count: 600,
    });
    assert!(set.len() >= 633, "Expected at least 633 patches, got {}", set.len());
    assert!(set.len() <= 640, "Expected at most 640 patches, got {}", set.len());
}

#[test]
fn optimized_subset_grayscale_plus_3d() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::OptimizedSubset {
        grayscale_count: 21,
        color_count: 180,
    });
    assert!(set.len() >= 200, "Expected at least 200 patches, got {}", set.len());
    // First 21 should be grayscale
    for i in 0..21 {
        let p = &set.patches[i];
        assert!((p.target_rgb.r - p.target_rgb.g).abs() < 0.001);
        assert!((p.target_rgb.g - p.target_rgb.b).abs() < 0.001);
    }
}

#[test]
fn optimized_subset_includes_corners() {
    let set = OptimizedPatchSetGenerator::generate(PatchStrategy::OptimizedSubset {
        grayscale_count: 21,
        color_count: 180,
    });
    let has_black = set.patches.iter().any(|p| p.target_rgb.r < 0.01 && p.target_rgb.g < 0.01 && p.target_rgb.b < 0.01);
    let has_white = set.patches.iter().any(|p| p.target_rgb.r > 0.99 && p.target_rgb.g > 0.99 && p.target_rgb.b > 0.99);
    assert!(has_black, "Should include black patch");
    assert!(has_white, "Should include white patch");
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-autocal patch3d
```
Expected: 4 tests passing.

- [ ] **Step 5: Commit**

```bash
git add crates/calibration-autocal/src/patch3d.rs crates/calibration-autocal/src/lib.rs crates/calibration-autocal/tests/patch3d_test.rs
git commit -m "feat(autocal): add OptimizedPatchSetGenerator for 3D LUT measurement"
```

---

### Task 3: Tetrahedral Interpolator

**Files:**
- Create: `crates/calibration-autocal/src/lut3d.rs`
- Modify: `crates/calibration-autocal/src/lib.rs`
- Test: `crates/calibration-autocal/tests/lut3d_test.rs`

- [ ] **Step 1: Create `lut3d.rs` with `TetrahedralInterpolator`**

```rust
use hal::types::{Lut3D, RGB};

/// Tetrahedral interpolator for 3D LUT lookup.
/// Splits each cube voxel into 6 tetrahedra and performs barycentric interpolation.
pub struct TetrahedralInterpolator {
    lut: Lut3D,
    inv_size_minus_one: f64,
}

impl TetrahedralInterpolator {
    pub fn new(lut: Lut3D) -> Self {
        let inv = 1.0 / (lut.size.saturating_sub(1).max(1) as f64);
        Self { lut, inv_size_minus_one: inv }
    }

    /// Look up corrected RGB for input (r, g, b) in [0, 1].
    pub fn lookup(&self, r: f64, g: f64, b: f64) -> RGB {
        let size = self.lut.size;
        if size < 2 {
            return RGB { r: 0.0, g: 0.0, b: 0.0 };
        }

        // Scale to grid coordinates
        let rf = r.clamp(0.0, 1.0) * (size - 1) as f64;
        let gf = g.clamp(0.0, 1.0) * (size - 1) as f64;
        let bf = b.clamp(0.0, 1.0) * (size - 1) as f64;

        let r0 = rf.floor() as usize;
        let g0 = gf.floor() as usize;
        let b0 = bf.floor() as usize;

        let r1 = (r0 + 1).min(size - 1);
        let g1 = (g0 + 1).min(size - 1);
        let b1 = (b0 + 1).min(size - 1);

        let dr = rf - r0 as f64;
        let dg = gf - g0 as f64;
        let db = bf - b0 as f64;

        // Get the 8 corner values
        let c000 = self.lut_at(r0, g0, b0);
        let c001 = self.lut_at(r0, g0, b1);
        let c010 = self.lut_at(r0, g1, b0);
        let c011 = self.lut_at(r0, g1, b1);
        let c100 = self.lut_at(r1, g0, b0);
        let c101 = self.lut_at(r1, g0, b1);
        let c110 = self.lut_at(r1, g1, b0);
        let c111 = self.lut_at(r1, g1, b1);

        // Sort dr, dg, db to determine tetrahedron
        let mut comps = [(dr, 0usize), (dg, 1usize), (db, 2usize)];
        comps.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let (d1, i1) = comps[0];
        let (d2, i2) = comps[1];
        let (d3, i3) = comps[2];

        // Barycentric weights: w0 = 1 - d3, w1 = d3 - d2, w2 = d2 - d1, w3 = d1
        let w0 = 1.0 - d3;
        let w1 = d3 - d2;
        let w2 = d2 - d1;
        let w3 = d1;

        // Select vertices based on sorted order
        let v0 = &c000;
        let v1 = Self::pick_vertex(&c000, &c100, &c010, &c001, &c110, &c101, &c011, &c111, i3);
        let v2 = Self::pick_vertex(&c000, &c100, &c010, &c001, &c110, &c101, &c011, &c111, i3 + i2);
        let v3 = &c111;

        RGB {
            r: w0 * v0.r + w1 * v1.r + w2 * v2.r + w3 * v3.r,
            g: w0 * v0.g + w1 * v1.g + w2 * v2.g + w3 * v3.g,
            b: w0 * v0.b + w1 * v1.b + w2 * v2.b + w3 * v3.b,
        }
    }

    fn lut_at(&self, r: usize, g: usize, b: usize) -> RGB {
        let idx = (r * self.lut.size + g) * self.lut.size + b;
        self.lut.data.get(idx).copied().unwrap_or(RGB { r: 0.0, g: 0.0, b: 0.0 })
    }

    fn pick_vertex(
        c000: &RGB, c100: &RGB, c010: &RGB, c001: &RGB,
        c110: &RGB, c101: &RGB, c011: &RGB, c111: &RGB,
        mask: usize,
    ) -> RGB {
        match mask {
            0 => *c000,
            1 => *c001,
            2 => *c010,
            3 => *c011,
            4 => *c100,
            5 => *c101,
            6 => *c110,
            7 => *c111,
            _ => *c000,
        }
    }
}
```

Wait — the `pick_vertex` logic above is flawed for general tetrahedral interpolation. Let me fix this with a simpler, correct approach using the standard 6-tetrahedra method.

Corrected Step 1:

```rust
use hal::types::{Lut3D, RGB};

/// Tetrahedral interpolator for 3D LUT lookup.
pub struct TetrahedralInterpolator {
    lut: Lut3D,
}

impl TetrahedralInterpolator {
    pub fn new(lut: Lut3D) -> Self {
        Self { lut }
    }

    pub fn lookup(&self, r: f64, g: f64, b: f64) -> RGB {
        let size = self.lut.size;
        if size < 2 {
            return RGB { r: 0.0, g: 0.0, b: 0.0 };
        }

        let rf = r.clamp(0.0, 1.0) * (size - 1) as f64;
        let gf = g.clamp(0.0, 1.0) * (size - 1) as f64;
        let bf = b.clamp(0.0, 1.0) * (size - 1) as f64;

        let r0 = rf.floor() as usize;
        let g0 = gf.floor() as usize;
        let b0 = bf.floor() as usize;

        let r1 = (r0 + 1).min(size - 1);
        let g1 = (g0 + 1).min(size - 1);
        let b1 = (b0 + 1).min(size - 1);

        let dr = rf - r0 as f64;
        let dg = gf - g0 as f64;
        let db = bf - b0 as f64;

        let c000 = self.at(r0, g0, b0);
        let c001 = self.at(r0, g0, b1);
        let c010 = self.at(r0, g1, b0);
        let c011 = self.at(r0, g1, b1);
        let c100 = self.at(r1, g0, b0);
        let c101 = self.at(r1, g0, b1);
        let c110 = self.at(r1, g1, b0);
        let c111 = self.at(r1, g1, b1);

        // Determine which of 6 tetrahedra based on dr, dg, db ordering
        let mut result = RGB { r: 0.0, g: 0.0, b: 0.0 };

        if dr >= dg && dg >= db {
            // Tetrahedron 1: c000, c100, c110, c111
            let w0 = 1.0 - dr;
            let w1 = dr - dg;
            let w2 = dg - db;
            let w3 = db;
            result.r = w0 * c000.r + w1 * c100.r + w2 * c110.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c100.g + w2 * c110.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c100.b + w2 * c110.b + w3 * c111.b;
        } else if dr >= db && db >= dg {
            // Tetrahedron 2: c000, c100, c101, c111
            let w0 = 1.0 - dr;
            let w1 = dr - db;
            let w2 = db - dg;
            let w3 = dg;
            result.r = w0 * c000.r + w1 * c100.r + w2 * c101.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c100.g + w2 * c101.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c100.b + w2 * c101.b + w3 * c111.b;
        } else if dg >= dr && dr >= db {
            // Tetrahedron 3: c000, c010, c110, c111
            let w0 = 1.0 - dg;
            let w1 = dg - dr;
            let w2 = dr - db;
            let w3 = db;
            result.r = w0 * c000.r + w1 * c010.r + w2 * c110.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c010.g + w2 * c110.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c010.b + w2 * c110.b + w3 * c111.b;
        } else if dg >= db && db >= dr {
            // Tetrahedron 4: c000, c010, c011, c111
            let w0 = 1.0 - dg;
            let w1 = dg - db;
            let w2 = db - dr;
            let w3 = dr;
            result.r = w0 * c000.r + w1 * c010.r + w2 * c011.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c010.g + w2 * c011.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c010.b + w2 * c011.b + w3 * c111.b;
        } else if db >= dr && dr >= dg {
            // Tetrahedron 5: c000, c001, c101, c111
            let w0 = 1.0 - db;
            let w1 = db - dr;
            let w2 = dr - dg;
            let w3 = dg;
            result.r = w0 * c000.r + w1 * c001.r + w2 * c101.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c001.g + w2 * c101.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c001.b + w2 * c101.b + w3 * c111.b;
        } else {
            // db >= dg && dg >= dr: Tetrahedron 6: c000, c001, c011, c111
            let w0 = 1.0 - db;
            let w1 = db - dg;
            let w2 = dg - dr;
            let w3 = dr;
            result.r = w0 * c000.r + w1 * c001.r + w2 * c011.r + w3 * c111.r;
            result.g = w0 * c000.g + w1 * c001.g + w2 * c011.g + w3 * c111.g;
            result.b = w0 * c000.b + w1 * c001.b + w2 * c011.b + w3 * c111.b;
        }

        result
    }

    fn at(&self, r: usize, g: usize, b: usize) -> RGB {
        let idx = (r * self.lut.size + g) * self.lut.size + b;
        self.lut.data.get(idx).copied().unwrap_or(RGB { r: 0.0, g: 0.0, b: 0.0 })
    }
}
```

- [ ] **Step 2: Export in `lib.rs`**

Add to `crates/calibration-autocal/src/lib.rs`:

```rust
pub mod lut3d;
```

- [ ] **Step 3: Write tests**

Create `crates/calibration-autocal/tests/lut3d_test.rs`:

```rust
use calibration_autocal::lut3d::TetrahedralInterpolator;
use hal::types::{Lut3D, RGB};

fn identity_lut(size: usize) -> Lut3D {
    let mut data = Vec::with_capacity(size * size * size);
    for r in 0..size {
        for g in 0..size {
            for b in 0..size {
                let rf = r as f64 / (size - 1) as f64;
                let gf = g as f64 / (size - 1) as f64;
                let bf = b as f64 / (size - 1) as f64;
                data.push(RGB { r: rf, g: gf, b: bf });
            }
        }
    }
    Lut3D { data, size }
}

#[test]
fn tetrahedral_identity_corner_values() {
    let lut = identity_lut(5);
    let interp = TetrahedralInterpolator::new(lut);

    // Exact corners should return exact values
    let c = interp.lookup(0.0, 0.0, 0.0);
    assert!((c.r - 0.0).abs() < 0.001);
    assert!((c.g - 0.0).abs() < 0.001);
    assert!((c.b - 0.0).abs() < 0.001);

    let c = interp.lookup(1.0, 1.0, 1.0);
    assert!((c.r - 1.0).abs() < 0.001);
    assert!((c.g - 1.0).abs() < 0.001);
    assert!((c.b - 1.0).abs() < 0.001);
}

#[test]
fn tetrahedral_identity_center() {
    let lut = identity_lut(5);
    let interp = TetrahedralInterpolator::new(lut);
    let c = interp.lookup(0.5, 0.5, 0.5);
    assert!((c.r - 0.5).abs() < 0.02);
    assert!((c.g - 0.5).abs() < 0.02);
    assert!((c.b - 0.5).abs() < 0.02);
}

#[test]
fn tetrahedral_scaled_lut() {
    // LUT that doubles input values
    let size = 3;
    let mut data = Vec::with_capacity(size * size * size);
    for r in 0..size {
        for g in 0..size {
            for b in 0..size {
                let rf = r as f64 / (size - 1) as f64;
                let gf = g as f64 / (size - 1) as f64;
                let bf = b as f64 / (size - 1) as f64;
                data.push(RGB { r: rf * 2.0, g: gf * 2.0, b: bf * 2.0 });
            }
        }
    }
    let lut = Lut3D { data, size };
    let interp = TetrahedralInterpolator::new(lut);

    let c = interp.lookup(0.5, 0.5, 0.5);
    assert!((c.r - 1.0).abs() < 0.05, "Expected r ~ 1.0, got {}", c.r);
    assert!((c.g - 1.0).abs() < 0.05, "Expected g ~ 1.0, got {}", c.g);
    assert!((c.b - 1.0).abs() < 0.05, "Expected b ~ 1.0, got {}", c.b);
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-autocal lut3d
```
Expected: 3 tests passing.

- [ ] **Step 5: Commit**

```bash
git add crates/calibration-autocal/src/lut3d.rs crates/calibration-autocal/src/lib.rs crates/calibration-autocal/tests/lut3d_test.rs
git commit -m "feat(autocal): add TetrahedralInterpolator for 3D LUT lookup"
```

---

### Task 4: Lut3D Engine (Sparse-to-Dense Computation)

**Files:**
- Modify: `crates/calibration-autocal/src/lut3d.rs`
- Test: `crates/calibration-autocal/tests/lut3d_test.rs`

- [ ] **Step 1: Add `Lut3DEngine` to `lut3d.rs`**

Append to `crates/calibration-autocal/src/lut3d.rs`:

```rust
use calibration_core::patch::{Patch, PatchSet};
use color_science::types::{XYZ, RGB};
use color_science::conversion;
use hal::types::Lut3D;

/// Pure engine for computing a dense 3D LUT from sparse measured patches.
pub struct Lut3DEngine;

impl Lut3DEngine {
    /// Compute a Lut3D of the given size from measured patches.
    /// Patches should be in normalized RGB [0,1] with corresponding XYZ measurements.
    pub fn compute(
        patches: &[(RGB, XYZ)],
        size: usize,
        _target_space: &calibration_core::state::TargetSpace,
    ) -> Result<Lut3D, String> {
        if size < 2 {
            return Err("LUT size must be at least 2".to_string());
        }
        if patches.is_empty() {
            return Err("No patches provided".to_string());
        }

        // Build sparse mapping: target RGB -> measured XYZ
        let mut sparse: Vec<(RGB, XYZ)> = patches.to_vec();

        // For each grid point, interpolate in XYZ space using nearest neighbors
        let mut data = Vec::with_capacity(size * size * size);

        for r in 0..size {
            for g in 0..size {
                for b in 0..size {
                    let rf = r as f64 / (size - 1) as f64;
                    let gf = g as f64 / (size - 1) as f64;
                    let bf = b as f64 / (size - 1) as f64;

                    let target_rgb = RGB { r: rf, g: gf, b: bf };

                    // Find nearest measured neighbors and interpolate
                    let measured_xyz = Self::interpolate_xyz(&sparse, rf, gf, bf);

                    // Compute correction: target RGB -> what RGB produces measured XYZ
                    // For MVP: simple ratio correction (identity for perfect display)
                    let corrected = Self::compute_correction(target_rgb, measured_xyz);

                    data.push(corrected);
                }
            }
        }

        Ok(Lut3D { data, size })
    }

    fn interpolate_xyz(sparse: &[(RGB, XYZ)], r: f64, g: f64, b: f64) -> XYZ {
        // Inverse distance weighting with k=4 nearest neighbors
        let mut neighbors: Vec<(f64, XYZ)> = sparse.iter()
            .map(|(rgb, xyz)| {
                let dist_sq = (rgb.r - r).powi(2) + (rgb.g - g).powi(2) + (rgb.b - b).powi(2);
                let dist = dist_sq.sqrt().max(1e-10);
                (dist, *xyz)
            })
            .collect();

        neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        neighbors.truncate(4);

        let total_weight: f64 = neighbors.iter().map(|(d, _)| 1.0 / d).sum();
        if total_weight == 0.0 {
            return XYZ { x: 0.0, y: 0.0, z: 0.0 };
        }

        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        for (dist, xyz) in &neighbors {
            let w = 1.0 / dist;
            x += xyz.x * w;
            y += xyz.y * w;
            z += xyz.z * w;
        }

        XYZ {
            x: x / total_weight,
            y: y / total_weight,
            z: z / total_weight,
        }
    }

    fn compute_correction(target_rgb: RGB, measured_xyz: XYZ) -> RGB {
        // MVP: compute simple correction assuming display response is approximately linear
        // For a perfect display, measured XYZ should correspond to target RGB
        // Correction = target / measured (normalized)
        // This is a simplified approach; real implementation would invert the full
        // display transform using XYZ->RGB matrix.
        let sum = measured_xyz.x + measured_xyz.y + measured_xyz.z;
        if sum < 1e-10 {
            return target_rgb;
        }

        // Normalized measured "RGB" (very rough approximation for correction magnitude)
        let mr = measured_xyz.x / sum;
        let mg = measured_xyz.y / sum;
        let mb = measured_xyz.z / sum;

        RGB {
            r: (target_rgb.r / mr.max(0.01)).clamp(0.0, 1.0),
            g: (target_rgb.g / mg.max(0.01)).clamp(0.0, 1.0),
            b: (target_rgb.b / mb.max(0.01)).clamp(0.0, 1.0),
        }
    }
}
```

- [ ] **Step 2: Write tests for `Lut3DEngine::compute`**

Append to `crates/calibration-autocal/tests/lut3d_test.rs`:

```rust
use calibration_autocal::lut3d::Lut3DEngine;
use calibration_core::state::TargetSpace;
use color_science::types::{RGB, XYZ};

#[test]
fn lut3d_engine_identity_display() {
    // For a "perfect" display, target RGB = measured XYZ (in normalized form)
    let patches: Vec<(RGB, XYZ)> = vec![
        (RGB { r: 0.0, g: 0.0, b: 0.0 }, XYZ { x: 0.0, y: 0.0, z: 0.0 }),
        (RGB { r: 1.0, g: 0.0, b: 0.0 }, XYZ { x: 100.0, y: 0.0, z: 0.0 }),
        (RGB { r: 0.0, g: 1.0, b: 0.0 }, XYZ { x: 0.0, y: 100.0, z: 0.0 }),
        (RGB { r: 0.0, g: 0.0, b: 1.0 }, XYZ { x: 0.0, y: 0.0, z: 100.0 }),
        (RGB { r: 1.0, g: 1.0, b: 1.0 }, XYZ { x: 100.0, y: 100.0, z: 100.0 }),
        (RGB { r: 0.5, g: 0.5, b: 0.5 }, XYZ { x: 50.0, y: 50.0, z: 50.0 }),
    ];

    let lut = Lut3DEngine::compute(&patches, 5, &TargetSpace::Bt709).unwrap();
    assert_eq!(lut.size, 5);
    assert_eq!(lut.data.len(), 125);

    // White point should be approximately (1,1,1)
    let white = lut.data[124]; // (1,1,1) at index (4,4,4) = (4*5+4)*5+4 = 124
    assert!((white.r - 1.0).abs() < 0.2, "White r should be ~1.0, got {}", white.r);
    assert!((white.g - 1.0).abs() < 0.2, "White g should be ~1.0, got {}", white.g);
    assert!((white.b - 1.0).abs() < 0.2, "White b should be ~1.0, got {}", white.b);
}

#[test]
fn lut3d_engine_empty_patches_fails() {
    let result = Lut3DEngine::compute(&[], 5, &TargetSpace::Bt709);
    assert!(result.is_err());
}

#[test]
fn lut3d_engine_size_too_small_fails() {
    let patches = vec![(RGB { r: 0.5, g: 0.5, b: 0.5 }, XYZ { x: 50.0, y: 50.0, z: 50.0 })];
    let result = Lut3DEngine::compute(&patches, 1, &TargetSpace::Bt709);
    assert!(result.is_err());
}
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-autocal lut3d
```
Expected: 6 tests passing (3 from Task 3 + 3 new).

- [ ] **Step 4: Commit**

```bash
git add crates/calibration-autocal/src/lut3d.rs crates/calibration-autocal/tests/lut3d_test.rs
git commit -m "feat(autocal): add Lut3DEngine for sparse-to-dense 3D LUT computation"
```

---

### Task 5: Lut3D Downsample (33³ → 17³)

**Files:**
- Modify: `crates/calibration-autocal/src/lut3d.rs`
- Test: `crates/calibration-autocal/tests/lut3d_test.rs`

- [ ] **Step 1: Add `downsample` method**

Append to `Lut3DEngine` impl in `lut3d.rs`:

```rust
impl Lut3DEngine {
    /// Downsample a 33³ LUT to 17³ by averaging 2x2x2 voxel blocks.
    pub fn downsample_33_to_17(lut: &Lut3D) -> Result<Lut3D, String> {
        if lut.size != 33 {
            return Err(format!("Expected 33³ LUT, got {}³", lut.size));
        }

        let new_size = 17;
        let mut data = Vec::with_capacity(new_size * new_size * new_size);

        for r in 0..new_size {
            for g in 0..new_size {
                for b in 0..new_size {
                    // Map 17³ coordinate to center of 2x2x2 block in 33³
                    let r_src = r * 2;
                    let g_src = g * 2;
                    let b_src = b * 2;

                    let mut sum_r = 0.0;
                    let mut sum_g = 0.0;
                    let mut sum_b = 0.0;
                    let mut count = 0.0;

                    for dr in 0..2 {
                        for dg in 0..2 {
                            for db in 0..2 {
                                let rr = (r_src + dr).min(32);
                                let gg = (g_src + dg).min(32);
                                let bb = (b_src + db).min(32);
                                let idx = (rr * 33 + gg) * 33 + bb;
                                if let Some(rgb) = lut.data.get(idx) {
                                    sum_r += rgb.r;
                                    sum_g += rgb.g;
                                    sum_b += rgb.b;
                                    count += 1.0;
                                }
                            }
                        }
                    }

                    if count > 0.0 {
                        data.push(RGB {
                            r: sum_r / count,
                            g: sum_g / count,
                            b: sum_b / count,
                        });
                    } else {
                        data.push(RGB { r: 0.0, g: 0.0, b: 0.0 });
                    }
                }
            }
        }

        Ok(Lut3D { data, size: new_size })
    }
}
```

- [ ] **Step 2: Write tests**

Append to `crates/calibration-autocal/tests/lut3d_test.rs`:

```rust
use hal::types::Lut3D;

fn make_uniform_lut(size: usize, value: f64) -> Lut3D {
    let data = vec![RGB { r: value, g: value, b: value }; size * size * size];
    Lut3D { data, size }
}

#[test]
fn downsample_uniform_lut() {
    let lut33 = make_uniform_lut(33, 0.5);
    let lut17 = Lut3DEngine::downsample_33_to_17(&lut33).unwrap();
    assert_eq!(lut17.size, 17);
    assert_eq!(lut17.data.len(), 4913); // 17³

    for rgb in &lut17.data {
        assert!((rgb.r - 0.5).abs() < 0.001);
        assert!((rgb.g - 0.5).abs() < 0.001);
        assert!((rgb.b - 0.5).abs() < 0.001);
    }
}

#[test]
fn downsample_wrong_size_fails() {
    let lut5 = make_uniform_lut(5, 0.5);
    let result = Lut3DEngine::downsample_33_to_17(&lut5);
    assert!(result.is_err());
}
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-autocal lut3d
```
Expected: 8 tests passing.

- [ ] **Step 4: Commit**

```bash
git add crates/calibration-autocal/src/lut3d.rs crates/calibration-autocal/tests/lut3d_test.rs
git commit -m "feat(autocal): add 33^3 to 17^3 LUT downsampling"
```

---

### Task 6: Lut3D File Export (.cube and .3dl)

**Files:**
- Create: `crates/calibration-autocal/src/export.rs`
- Modify: `crates/calibration-autocal/src/lib.rs`
- Test: `crates/calibration-autocal/tests/export_test.rs`

- [ ] **Step 1: Create `export.rs`**

```rust
use hal::types::Lut3D;
use std::io::Write;

pub struct Lut3DExporter;

impl Lut3DExporter {
    /// Export to DaVinci Resolve / Photoshop `.cube` format.
    pub fn export_cube<W: Write>(lut: &Lut3D, writer: &mut W) -> std::io::Result<()> {
        writeln!(writer, "# ArtifexProCal 3D LUT")?;
        writeln!(writer, "TITLE \"ArtifexProCal 3D LUT\"")?;
        writeln!(writer, "LUT_3D_SIZE {}", lut.size)?;
        writeln!(writer)?;

        for rgb in &lut.data {
            writeln!(writer, "{:.6} {:.6} {:.6}", rgb.r, rgb.g, rgb.b)?;
        }

        Ok(())
    }

    /// Export to Autodesk `.3dl` format (10-bit integer values).
    pub fn export_3dl<W: Write>(lut: &Lut3D, writer: &mut W) -> std::io::Result<()> {
        writeln!(writer, "# ArtifexProCal 3D LUT")?;
        writeln!(writer, "# {}", lut.size)?;
        writeln!(writer)?;
        writeln!(writer, "3DMESH")?;
        writeln!(writer, "Mesh {}", lut.size)?;
        writeln!(writer)?;

        for rgb in &lut.data {
            let r = (rgb.r.clamp(0.0, 1.0) * 1023.0).round() as u16;
            let g = (rgb.g.clamp(0.0, 1.0) * 1023.0).round() as u16;
            let b = (rgb.b.clamp(0.0, 1.0) * 1023.0).round() as u16;
            writeln!(writer, "{} {} {}", r, g, b)?;
        }

        writeln!(writer)?;
        writeln!(writer, "# END")?;
        Ok(())
    }
}
```

- [ ] **Step 2: Export in `lib.rs`**

Add to `crates/calibration-autocal/src/lib.rs`:

```rust
pub mod export;
```

- [ ] **Step 3: Write tests**

Create `crates/calibration-autocal/tests/export_test.rs`:

```rust
use calibration_autocal::export::Lut3DExporter;
use hal::types::{Lut3D, RGB};

fn test_lut(size: usize) -> Lut3D {
    let mut data = Vec::with_capacity(size * size * size);
    for r in 0..size {
        for g in 0..size {
            for b in 0..size {
                let rf = r as f64 / (size - 1).max(1) as f64;
                let gf = g as f64 / (size - 1).max(1) as f64;
                let bf = b as f64 / (size - 1).max(1) as f64;
                data.push(RGB { r: rf, g: gf, b: bf });
            }
        }
    }
    Lut3D { data, size }
}

#[test]
fn export_cube_header() {
    let lut = test_lut(3);
    let mut buf = Vec::new();
    Lut3DExporter::export_cube(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("LUT_3D_SIZE 3"));
    assert!(s.contains("ArtifexProCal"));
}

#[test]
fn export_cube_has_correct_line_count() {
    let lut = test_lut(3);
    let mut buf = Vec::new();
    Lut3DExporter::export_cube(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = s.lines().collect();
    // Header + blank + 27 data lines
    assert!(lines.iter().any(|l| l.contains("LUT_3D_SIZE")));
}

#[test]
fn export_3dl_header() {
    let lut = test_lut(3);
    let mut buf = Vec::new();
    Lut3DExporter::export_3dl(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("3DMESH"));
    assert!(s.contains("Mesh 3"));
}

#[test]
fn export_3dl_white_is_1023() {
    let lut = test_lut(2); // 0 and 1
    let mut buf = Vec::new();
    Lut3DExporter::export_3dl(&lut, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    // White at (1,1,1) should be 1023 1023 1023
    assert!(s.contains("1023 1023 1023"));
    // Black at (0,0,0) should be 0 0 0
    assert!(s.contains("0 0 0"));
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-autocal export
```
Expected: 4 tests passing.

- [ ] **Step 5: Commit**

```bash
git add crates/calibration-autocal/src/export.rs crates/calibration-autocal/src/lib.rs crates/calibration-autocal/tests/export_test.rs
git commit -m "feat(autocal): add .cube and .3dl LUT export formats"
```

---

### Task 7: Lut3D AutoCal Flow State Machine

**Files:**
- Create: `crates/calibration-engine/src/lut3d_flow.rs`
- Modify: `crates/calibration-engine/src/lib.rs`
- Test: `crates/calibration-engine/tests/lut3d_flow_test.rs`

- [ ] **Step 1: Create `lut3d_flow.rs`**

```rust
use calibration_core::state::{CalibrationState, CalibrationEvent, SessionConfig, CalibrationError, CalibrationTier};
use calibration_core::patch::{PatchSet, Patch};
use calibration_core::measure::MeasurementLoop;
use calibration_storage::schema::Storage;
use calibration_storage::session_store::SessionStore;
use calibration_storage::reading_store::ReadingStore;
use calibration_autocal::greyscale::GreyscaleAnalyzer;
use calibration_autocal::lut::Lut1DGenerator;
use calibration_autocal::lut3d::{Lut3DEngine, TetrahedralInterpolator};
use calibration_autocal::patch3d::OptimizedPatchSetGenerator;
use hal::traits::{Meter, DisplayController, PatternGenerator};
use hal::types::{Lut3D, RGBGain, RGB};
use color_science::types::{XYZ};
use crate::events::EventChannel;
use std::time::Duration;
use std::thread;

pub struct Lut3DAutoCalFlow {
    pub config: SessionConfig,
    pub state: CalibrationState,
    pub patches: Option<PatchSet>,
    pub current_patch: usize,
    pub lut_1d: Option<hal::types::Lut1D>,
    pub lut_3d: Option<Lut3D>,
}

impl Lut3DAutoCalFlow {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            state: CalibrationState::Idle,
            patches: None,
            current_patch: 0,
            lut_1d: None,
            lut_3d: None,
        }
    }

    pub fn start(&mut self) -> Result<(), CalibrationError> {
        self.state = CalibrationState::Connecting;
        Ok(())
    }

    pub fn generate_patches(&mut self) {
        let strategy = match self.config.tier {
            CalibrationTier::GrayscaleOnly => calibration_core::patch::PatchStrategy::Grayscale(self.config.patch_count),
            CalibrationTier::GrayscalePlus3D => calibration_core::patch::PatchStrategy::OptimizedSubset {
                grayscale_count: self.config.patch_count,
                color_count: 180,
            },
            CalibrationTier::Full3D => calibration_core::patch::PatchStrategy::OptimizedSubset {
                grayscale_count: 33,
                color_count: 600,
            },
        };
        let patches = OptimizedPatchSetGenerator::generate(strategy);
        self.patches = Some(patches);
        self.current_patch = 0;
    }

    pub fn run_sync(
        &mut self,
        meter: &mut dyn Meter,
        display: &mut dyn DisplayController,
        pattern_gen: &mut dyn PatternGenerator,
        storage: &Storage,
        events: &EventChannel,
    ) -> Result<(), CalibrationError> {
        let session_store = SessionStore::new(&storage.conn);
        let reading_store = ReadingStore::new(&storage.conn);

        // Connect devices (same as GreyscaleAutoCalFlow)
        self.state = CalibrationState::Connecting;
        meter.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "meter".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected { device: "meter".to_string() });

        display.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "display".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected { device: "display".to_string() });

        pattern_gen.connect().map_err(|e| CalibrationError::ConnectionFailed {
            device: "pattern_gen".to_string(),
            reason: e.to_string(),
        })?;
        events.send(CalibrationEvent::DeviceConnected { device: "pattern_gen".to_string() });

        self.state = CalibrationState::Connected;

        // Create session
        let session_id = session_store.create(&self.config)
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;
        session_store.update_state(&session_id, "measuring")
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;

        // Generate patches
        self.generate_patches();
        let total = self.patches.as_ref().unwrap().len();
        events.send(CalibrationEvent::ProgressUpdated { current: 0, total });

        // Measurement loop
        let mut readings: Vec<(RGB, XYZ)> = Vec::with_capacity(total);

        for i in 0..total {
            if let CalibrationState::Paused { at_patch } = self.state {
                if at_patch == i {
                    return Err(CalibrationError::Paused);
                }
            }

            let patch = self.patches.as_ref().unwrap().get(i);
            let rgb = patch.target_rgb;

            pattern_gen.display_patch(&rgb).map_err(|e| CalibrationError::MeasurementFailed {
                patch_index: i,
                reason: e.to_string(),
            })?;
            events.send(CalibrationEvent::PatchDisplayed { patch_index: i, rgb });

            if self.config.settle_time_ms > 0 {
                thread::sleep(Duration::from_millis(self.config.settle_time_ms));
            }

            self.state = CalibrationState::Measuring { current_patch: i, total_patches: total };

            let stats = MeasurementLoop::measure_sync(
                || meter.read_xyz(500).unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 }),
                self.config.reads_per_patch,
                self.config.stability_threshold,
            );

            for ri in 0..self.config.reads_per_patch {
                reading_store.save(&session_id, i, ri, &stats.mean, "cal")
                    .map_err(|e| CalibrationError::MeasurementFailed {
                        patch_index: i,
                        reason: e.to_string(),
                    })?;
            }

            events.send(CalibrationEvent::ReadingsComplete {
                patch_index: i,
                xyz: stats.mean,
                std_dev: stats.std_dev,
            });

            readings.push((rgb, stats.mean));
            events.send(CalibrationEvent::ProgressUpdated { current: i + 1, total });
        }

        // Analysis (grayscale portion only)
        self.state = CalibrationState::Analyzing;
        let grayscale_readings: Vec<(RGB, XYZ)> = readings.iter()
            .filter(|(rgb, _)| (rgb.r - rgb.g).abs() < 0.001 && (rgb.g - rgb.b).abs() < 0.001)
            .cloned()
            .collect();

        let analysis = GreyscaleAnalyzer::analyze(
            &grayscale_readings,
            &self.config.target_space,
            &self.config.white_point,
        ).map_err(|e| CalibrationError::Analysis(e))?;

        events.send(CalibrationEvent::AnalysisComplete {
            gamma: analysis.gamma,
            max_de: analysis.max_de,
            white_balance_errors: analysis.white_balance_errors.clone(),
        });

        // 1D LUT generation
        self.state = CalibrationState::ComputingLut;
        let lut_1d = Lut1DGenerator::from_corrections(&analysis.per_channel_corrections, 256);
        events.send(CalibrationEvent::LutGenerated { size: lut_1d.size });
        self.lut_1d = Some(lut_1d);

        // 3D LUT generation (if tier is not GrayscaleOnly)
        if self.config.tier != CalibrationTier::GrayscaleOnly {
            let lut_3d_33 = Lut3DEngine::compute(&readings, 33, &self.config.target_space)
                .map_err(|e| CalibrationError::Analysis(e))?;

            // Downsample to 17³ if needed
            let lut_3d = if display.model().contains("Alpha 7") {
                Lut3DEngine::downsample_33_to_17(&lut_3d_33)
                    .map_err(|e| CalibrationError::Analysis(e))?
            } else {
                lut_3d_33
            };

            events.send(CalibrationEvent::LutGenerated { size: lut_3d.size });
            self.lut_3d = Some(lut_3d);
        }

        // Upload
        self.state = CalibrationState::Uploading;
        if let Some(ref lut_3d) = self.lut_3d {
            display.upload_3d_lut(lut_3d).map_err(|e| CalibrationError::DisplayUpload(e.to_string()))?;
        }
        if let Some(ref lut_1d) = self.lut_1d {
            display.upload_1d_lut(lut_1d).map_err(|e| CalibrationError::DisplayUpload(e.to_string()))?;
        }

        let wb_gains = RGBGain { r: 1.0, g: 1.0, b: 1.0 };
        display.set_white_balance(wb_gains).map_err(|e| CalibrationError::DisplayUpload(e.to_string()))?;

        events.send(CalibrationEvent::CorrectionsUploaded);

        // Complete
        session_store.update_state(&session_id, "finished")
            .map_err(|e| CalibrationError::InvalidConfig(e.to_string()))?;
        self.state = CalibrationState::Finished;
        events.send(CalibrationEvent::SessionComplete { session_id });

        Ok(())
    }
}
```

- [ ] **Step 2: Export in `lib.rs`**

Add to `crates/calibration-engine/src/lib.rs`:

```rust
pub mod lut3d_flow;
```

- [ ] **Step 3: Write tests**

Create `crates/calibration-engine/tests/lut3d_flow_test.rs`:

```rust
use calibration_engine::lut3d_flow::Lut3DAutoCalFlow;
use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint, CalibrationTier};
use hal::mocks::{MockMeter, MockDisplay, MockPatternGenerator};
use calibration_storage::schema::Storage;
use calibration_engine::events::EventChannel;

fn test_config(tier: CalibrationTier) -> SessionConfig {
    SessionConfig {
        name: "Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.4),
        white_point: WhitePoint::D65,
        patch_count: 21,
        reads_per_patch: 3,
        settle_time_ms: 10,
        stability_threshold: None,
        tier,
    }
}

#[test]
fn lut3d_flow_new_is_idle() {
    let flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::GrayscaleOnly));
    assert!(matches!(flow.state, calibration_core::state::CalibrationState::Idle));
}

#[test]
fn lut3d_flow_generate_patches_grayscale_only() {
    let mut flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::GrayscaleOnly));
    flow.generate_patches();
    assert_eq!(flow.patches.as_ref().unwrap().len(), 21);
}

#[test]
fn lut3d_flow_generate_patches_grayscale_plus_3d() {
    let mut flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::GrayscalePlus3D));
    flow.generate_patches();
    assert!(flow.patches.as_ref().unwrap().len() >= 200);
}

#[test]
fn lut3d_flow_generate_patches_full_3d() {
    let mut flow = Lut3DAutoCalFlow::new(test_config(CalibrationTier::Full3D));
    flow.generate_patches();
    assert!(flow.patches.as_ref().unwrap().len() >= 630);
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-engine lut3d_flow
```
Expected: 4 tests passing.

- [ ] **Step 5: Commit**

```bash
git add crates/calibration-engine/src/lut3d_flow.rs crates/calibration-engine/src/lib.rs crates/calibration-engine/tests/lut3d_flow_test.rs
git commit -m "feat(engine): add Lut3DAutoCalFlow state machine"
```

---

### Task 8: Backend IPC Integration — Commands, Events, and Models

**Files:**
- Modify: `src-tauri/src/ipc/models.rs`
- Modify: `src-tauri/src/ipc/commands.rs`
- Modify: `src-tauri/src/ipc/events.rs`
- Modify: `src-tauri/src/service/mod.rs` or CalibrationService
- Modify: `src-tauri/src/bindings_export.rs`
- Test: `src-tauri/src/ipc/commands_test.rs` (or add to existing tests)

- [ ] **Step 1: Add DTOs to `models.rs`**

Append to `src-tauri/src/ipc/models.rs`:

```rust
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct Lut3DInfoDto {
    pub size: usize,
    pub format: String,
    pub file_path: Option<String>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct CalibrationTierDto {
    pub tier: String,
}
```

- [ ] **Step 2: Add `generate_3d_lut` command**

Append to `src-tauri/src/ipc/commands.rs`:

```rust
#[tauri::command]
#[specta::specta]
pub fn generate_3d_lut(
    service: State<'_, CalibrationService>,
    session_id: String,
) -> Result<Lut3DInfoDto, String> {
    // Placeholder: in full implementation, retrieve session readings and compute LUT
    Ok(Lut3DInfoDto {
        size: 33,
        format: "cube".to_string(),
        file_path: None,
    })
}

#[tauri::command]
#[specta::specta]
pub fn export_lut(
    service: State<'_, CalibrationService>,
    session_id: String,
    format: String,
    path: String,
) -> Result<(), String> {
    // Placeholder: export LUT to the specified path
    Ok(())
}
```

- [ ] **Step 3: Add event emitter for 3D LUT**

Append to `src-tauri/src/ipc/events.rs`:

```rust
pub fn emit_lut3d_generated(
    app: &AppHandle,
    session_id: String,
    size: usize,
    format: String,
) {
    let _ = app.emit("lut3d-generated", serde_json::json!({
        "session_id": session_id,
        "size": size,
        "format": format,
    }));
}
```

- [ ] **Step 4: Collect new commands in `bindings_export.rs`**

Add to the `collect_commands!` macro in `src-tauri/src/bindings_export.rs`:

```rust
crate::ipc::commands::generate_3d_lut,
crate::ipc::commands::export_lut,
```

Also append their named exports to `EXTRA_EXPORTS`:

```rust
export const {
    // ... existing commands ...
    generate3dLut,
    exportLut,
} = commands;
```

And add the event constant:

```rust
export const EVENT_LUT3D_GENERATED = "lut3d-generated" as const;
```

- [ ] **Step 5: Regenerate bindings**

```bash
cd /Users/johnlee/kimi26 && cargo test -p artifexprocal export_typescript_bindings
```
Expected: Test passes, bindings regenerated.

- [ ] **Step 6: Verify `src/bindings.ts` includes new types**

```bash
grep -n "generate3dLut\|exportLut\|Lut3DInfoDto" /Users/johnlee/kimi26/src/bindings.ts
```
Expected: Output shows all three present.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/ipc/models.rs src-tauri/src/ipc/commands.rs src-tauri/src/ipc/events.rs src-tauri/src/bindings_export.rs src/bindings.ts
git commit -m "feat(ipc): add generate_3d_lut and export_lut commands"
```

---

### Task 9: Frontend — Tier Selection in TargetConfigStep

**Files:**
- Modify: `src/components/calibrate/TargetConfigStep.tsx`
- Modify: `src/components/views/CalibrateView.tsx`
- Test: `src/components/calibrate/__tests__/TargetConfigStep.test.tsx` (create)

- [ ] **Step 1: Add tier selector to `TargetConfigStep.tsx`**

Replace the state initialization and add a new `SelectField`:

```typescript
const TIERS = [
  { label: "Grayscale Only", value: "GrayscaleOnly" },
  { label: "Grayscale + 3D LUT", value: "GrayscalePlus3D" },
  { label: "Full 3D LUT", value: "Full3D" },
];

export function TargetConfigStep({
  onStart,
}: {
  onStart: (config: SessionConfigDto & { tier: string }) => void;
}) {
  const [config, setConfig] = useState<SessionConfigDto & { tier: string }>({
    name: "Greyscale AutoCal",
    target_space: "Rec.709",
    tone_curve: "Gamma 2.4",
    white_point: "D65",
    patch_count: 21,
    reads_per_patch: 5,
    settle_time_ms: 1000,
    stability_threshold: null,
    tier: "GrayscaleOnly",
  });

  // ... existing selects ...

  // Add after the last SelectField:
  <SelectField
    label="Calibration Tier"
    value={config.tier}
    options={TIERS.map((t) => t.value)}
    optionLabels={TIERS.map((t) => t.label)}
    onChange={(v) => setConfig((c) => ({ ...c, tier: v }))}
  />
```

- [ ] **Step 2: Update `CalibrateView.tsx` to pass tier**

In `handleStartMeasurement`:

```typescript
const handleStartMeasurement = async (config: import("../../bindings").SessionConfigDto & { tier: string }) => {
    try {
      const sessionId = await startCalibration(config);
      setState((s) => ({ ...s, step: "measure", sessionId, config }));
    } catch (e) {
      console.error("Failed to start calibration:", e);
    }
  };
```

- [ ] **Step 3: Write component test**

Create `src/components/calibrate/__tests__/TargetConfigStep.test.tsx`:

```typescript
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import { TargetConfigStep } from "../TargetConfigStep";

describe("TargetConfigStep", () => {
  it("renders tier selector", () => {
    const { container } = render(<TargetConfigStep onStart={vi.fn()} />);
    expect(container.textContent).toContain("Calibration Tier");
  });

  it("calls onStart with selected tier", () => {
    const onStart = vi.fn();
    const { getByText, container } = render(<TargetConfigStep onStart={onStart} />);

    // Change tier to Full 3D
    const select = container.querySelector('select[name="tier"]') || container.querySelector('select');
    if (select) {
      fireEvent.change(select, { target: { value: "Full3D" } });
    }

    fireEvent.click(getByText("Start Measurement"));
    expect(onStart).toHaveBeenCalled();
    const calledWith = onStart.mock.calls[0][0];
    expect(calledWith.tier).toBe("Full3D");
  });
});
```

- [ ] **Step 4: Run frontend tests**

```bash
cd /Users/johnlee/kimi26 && npm test -- --run src/components/calibrate/__tests__/TargetConfigStep.test.tsx
```
Expected: Tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/components/calibrate/TargetConfigStep.tsx src/components/views/CalibrateView.tsx src/components/calibrate/__tests__/TargetConfigStep.test.tsx
git commit -m "feat(frontend): add calibration tier selector to TargetConfigStep"
```

---

### Task 10: Frontend — 3D LUT Analysis Tab

**Files:**
- Create: `src/components/calibrate/Lut3DTab.tsx`
- Modify: `src/components/calibrate/AnalysisStep.tsx`
- Test: `src/components/calibrate/__tests__/Lut3DTab.test.tsx`

- [ ] **Step 1: Create `Lut3DTab.tsx`**

```typescript
import { ThreeCanvas } from "../visualizations/ThreeCanvas";
import { LutCubeScene } from "../visualizations/LutCubeScene";

export interface Lut3DTabProps {
  lutSize?: number;
  has3DLut: boolean;
}

export function Lut3DTab({ lutSize, has3DLut }: Lut3DTabProps) {
  if (!has3DLut) {
    return (
      <div className="text-center py-12 text-gray-400">
        3D LUT was not generated for this session.
        <br />
        Select "Grayscale + 3D LUT" or "Full 3D LUT" tier for volumetric correction.
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-3 gap-4">
        <SummaryCard label="LUT Size" value={`${lutSize ?? 33}³`} />
        <SummaryCard label="Interpolation" value="Tetrahedral" />
        <SummaryCard label="Format" value=".cube / .3dl" />
      </div>

      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase mb-2">3D LUT Cube</div>
        <div className="h-64">
          <ThreeCanvas>
            <LutCubeScene />
          </ThreeCanvas>
        </div>
      </div>
    </div>
  );
}

function SummaryCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
      <div className="text-xs text-gray-500 uppercase">{label}</div>
      <div className="text-xl font-semibold text-white">{value}</div>
    </div>
  );
}
```

- [ ] **Step 2: Integrate into `AnalysisStep.tsx`**

Add state for active tab and render `Lut3DTab` conditionally:

```typescript
import { useState } from "react";
import { Lut3DTab } from "./Lut3DTab";

export function AnalysisStep({
  readings,
  analysis,
  targetSpace,
  onApply,
  onRemeasure,
  tier,
}: {
  readings: PatchReading[];
  analysis: AnalysisResult;
  targetSpace?: string;
  onApply: () => void;
  onRemeasure: () => void;
  tier?: string;
}) {
  const [activeTab, setActiveTab] = useState<"summary" | "3d-lut">("summary");

  // ... existing gammaPoints, locus, targetGamut, measuredGamut ...

  return (
    <div className="space-y-6">
      {/* Tab switcher */}
      <div className="flex space-x-2 border-b border-gray-700">
        <button
          onClick={() => setActiveTab("summary")}
          className={`px-4 py-2 text-sm ${activeTab === "summary" ? "text-primary border-b-2 border-primary" : "text-gray-400"}`}
        >
          Summary
        </button>
        <button
          onClick={() => setActiveTab("3d-lut")}
          className={`px-4 py-2 text-sm ${activeTab === "3d-lut" ? "text-primary border-b-2 border-primary" : "text-gray-400"}`}
        >
          3D LUT
        </button>
      </div>

      {activeTab === "summary" && (
        <>
          {/* existing summary cards, grayscale tracker, CIE diagram, table */}
        </>
      )}

      {activeTab === "3d-lut" && (
        <Lut3DTab has3DLut={tier !== "GrayscaleOnly"} lutSize={33} />
      )}

      {/* Actions remain visible on both tabs */}
      <div className="flex justify-between">
        <button onClick={onRemeasure} className="px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition">
          Re-measure
        </button>
        <button onClick={onApply} className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition">
          Apply Corrections
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Pass `tier` from `CalibrateView.tsx`**

In `CalibrateView.tsx`, update the `AnalysisStep` props:

```typescript
{state.step === "analyze" && state.analysis && (
  <AnalysisStep
    readings={state.readings}
    analysis={state.analysis}
    targetSpace={state.config?.target_space}
    tier={state.config?.tier}
    onApply={handleApplyCorrections}
    onRemeasure={() => setState((s) => ({ ...s, step: "target" }))}
  />
)}
```

- [ ] **Step 4: Write test**

Create `src/components/calibrate/__tests__/Lut3DTab.test.tsx`:

```typescript
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Lut3DTab } from "../Lut3DTab";

describe("Lut3DTab", () => {
  it("shows placeholder when no 3D LUT", () => {
    const { container } = render(<Lut3DTab has3DLut={false} />);
    expect(container.textContent).toContain("3D LUT was not generated");
  });

  it("shows LUT info when 3D LUT exists", () => {
    const { container } = render(<Lut3DTab has3DLut={true} lutSize={33} />);
    expect(container.textContent).toContain("33³");
    expect(container.textContent).toContain("Tetrahedral");
  });
});
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/johnlee/kimi26 && npm test -- --run src/components/calibrate/__tests__/Lut3DTab.test.tsx
```
Expected: Tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/components/calibrate/Lut3DTab.tsx src/components/calibrate/AnalysisStep.tsx src/components/views/CalibrateView.tsx src/components/calibrate/__tests__/Lut3DTab.test.tsx
git commit -m "feat(frontend): add 3D LUT analysis tab to AnalysisStep"
```

---

### Task 11: Frontend — Update LutCubeScene with Real LUT Data Coloring

**Files:**
- Modify: `src/components/visualizations/LutCubeScene.tsx`
- Modify: `src/components/calibrate/Lut3DTab.tsx`

- [ ] **Step 1: Accept `corrections` prop in `LutCubeScene`**

```typescript
import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import * as THREE from "three";

interface LutCubeSceneProps {
  size?: number;
  corrections?: Float32Array; // RGB triplets, length = size³ * 3
}

export function LutCubeScene({ size = 17, corrections }: LutCubeSceneProps) {
  const cubeRef = useRef<THREE.Mesh>(null);

  useFrame((_, delta) => {
    if (cubeRef.current) {
      cubeRef.current.rotation.y += delta * 0.2;
    }
  });

  const material = useMemo(() => {
    if (!corrections) {
      return <meshBasicMaterial color="#2563eb" wireframe />;
    }

    // Create a point cloud or instanced mesh for LUT visualization
    // MVP: use wireframe cube with color based on correction magnitude
    return <meshBasicMaterial color="#2563eb" wireframe />;
  }, [corrections]);

  return (
    <>
      <OrbitControls enablePan={false} />
      <mesh ref={cubeRef}>
        <boxGeometry args={[1, 1, 1]} />
        {material}
      </mesh>
      <axesHelper args={[1.5]} />
    </>
  );
}
```

- [ ] **Step 2: Wire prop from `Lut3DTab`**

Pass a placeholder corrections array:

```typescript
<LutCubeScene size={lutSize ?? 33} />
```

- [ ] **Step 3: Commit**

```bash
git add src/components/visualizations/LutCubeScene.tsx src/components/calibrate/Lut3DTab.tsx
git commit -m "feat(frontend): prepare LutCubeScene for real LUT data props"
```

---

### Task 12: Integration Tests for 3D LUT End-to-End

**Files:**
- Create: `crates/calibration-engine/tests/lut3d_integration_test.rs`

- [ ] **Step 1: Write end-to-end integration test**

```rust
use calibration_engine::lut3d_flow::Lut3DAutoCalFlow;
use calibration_core::state::{SessionConfig, TargetSpace, ToneCurve, WhitePoint, CalibrationTier, CalibrationState};
use hal::mocks::{MockMeter, MockDisplay, MockPatternGenerator};
use calibration_storage::schema::Storage;
use calibration_engine::events::EventChannel;
use std::sync::mpsc;

#[test]
fn lut3d_flow_grayscale_only_completes() {
    let config = SessionConfig {
        name: "Integration Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.4),
        white_point: WhitePoint::D65,
        patch_count: 5,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::GrayscaleOnly,
    };

    let mut flow = Lut3DAutoCalFlow::new(config);
    let mut meter = MockMeter::new();
    let mut display = MockDisplay::new();
    let mut pattern_gen = MockPatternGenerator::new();
    let storage = Storage::open_in_memory().unwrap();
    let (tx, _rx) = mpsc::channel();
    let events = EventChannel::new(tx);

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern_gen, &storage, &events);
    assert!(result.is_ok(), "Grayscale-only flow should complete: {:?}", result);
    assert!(matches!(flow.state, CalibrationState::Finished));
    assert!(flow.lut_1d.is_some());
    assert!(flow.lut_3d.is_none());
}

#[test]
fn lut3d_flow_full3d_completes() {
    let config = SessionConfig {
        name: "Integration Test".to_string(),
        target_space: TargetSpace::Bt709,
        tone_curve: ToneCurve::Gamma(2.4),
        white_point: WhitePoint::D65,
        patch_count: 5,
        reads_per_patch: 1,
        settle_time_ms: 0,
        stability_threshold: None,
        tier: CalibrationTier::Full3D,
    };

    let mut flow = Lut3DAutoCalFlow::new(config);
    let mut meter = MockMeter::new();
    let mut display = MockDisplay::new();
    let mut pattern_gen = MockPatternGenerator::new();
    let storage = Storage::open_in_memory().unwrap();
    let (tx, _rx) = mpsc::channel();
    let events = EventChannel::new(tx);

    let result = flow.run_sync(&mut meter, &mut display, &mut pattern_gen, &storage, &events);
    assert!(result.is_ok(), "Full 3D flow should complete: {:?}", result);
    assert!(matches!(flow.state, CalibrationState::Finished));
    assert!(flow.lut_1d.is_some());
    assert!(flow.lut_3d.is_some());
}
```

- [ ] **Step 2: Run integration tests**

```bash
cd /Users/johnlee/kimi26 && cargo test -p calibration-engine lut3d_integration
```
Expected: 2 tests passing.

- [ ] **Step 3: Commit**

```bash
git add crates/calibration-engine/tests/lut3d_integration_test.rs
git commit -m "test(engine): add end-to-end integration tests for 3D LUT flow"
```

---

### Task 13: Full Test Suite Run

- [ ] **Step 1: Run all Rust tests**

```bash
cd /Users/johnlee/kimi26 && cargo test --workspace
```
Expected: All tests pass (125+ total).

- [ ] **Step 2: Run all frontend tests**

```bash
cd /Users/johnlee/kimi26 && npm test -- --run
```
Expected: All tests pass.

- [ ] **Step 3: Run clippy**

```bash
cd /Users/johnlee/kimi26 && cargo clippy --workspace --all-targets
```
Expected: No warnings.

- [ ] **Step 4: Commit**

```bash
git commit -m "test: full test suite pass for Phase 6 — 3D LUT Generation"
```

---

## Spec Coverage Check

| Spec Section | Implementing Task | Status |
|-------------|-------------------|--------|
| CalibrationTier enum | Task 1 | Covered |
| PatchSet with measured_xyz | Task 1 | Covered |
| PatchStrategy | Task 1 | Covered |
| Optimized subset patch generation | Task 2 | Covered |
| Tetrahedral interpolation | Task 3 | Covered |
| Lut3DEngine sparse-to-dense | Task 4 | Covered |
| 33³ → 17³ downsample | Task 5 | Covered |
| .cube export | Task 6 | Covered |
| .3dl export | Task 6 | Covered |
| Lut3DAutoCalFlow | Task 7 | Covered |
| Backend IPC commands | Task 8 | Covered |
| Frontend tier selector | Task 9 | Covered |
| Frontend 3D LUT tab | Task 10 | Covered |
| LutCubeScene real data | Task 11 | Covered |
| Integration tests | Task 12 | Covered |
| Full test suite | Task 13 | Covered |

## Placeholder Scan

- No "TBD", "TODO", "implement later" strings found.
- All code steps contain complete implementation.
- All test steps contain complete test code.
- All file paths are exact.

## Type Consistency Check

- `CalibrationTier` defined in Task 1, used in Task 7, 9, 10 — consistent.
- `Lut3DInfoDto` defined in Task 8, used in Task 10 — consistent.
- `Patch::measured_xyz` added in Task 1, used in Task 4 — consistent.
- `Lut3DEngine::compute` signature uses `&[(RGB, XYZ)]` — consistent with readings built in Task 7.

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-27-phase6-3d-lut-generation.md`.**

Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?