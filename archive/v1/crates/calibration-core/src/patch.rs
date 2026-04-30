use color_science::types::{RGB, XYZ};

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub target_rgb: RGB,
    pub measured_xyz: Option<XYZ>,
}

impl Patch {
    pub fn new(target_rgb: RGB) -> Self {
        Self {
            target_rgb,
            measured_xyz: None,
        }
    }

    pub fn with_measurement(target_rgb: RGB, measured_xyz: XYZ) -> Self {
        Self {
            target_rgb,
            measured_xyz: Some(measured_xyz),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatchSet {
    pub patches: Vec<Patch>,
}

impl PatchSet {
    pub fn len(&self) -> usize {
        self.patches.len()
    }

    pub fn get(&self, index: usize) -> &Patch {
        &self.patches[index]
    }

    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GreyscalePatchSet;

impl GreyscalePatchSet {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(count: usize) -> PatchSet {
        let mut patches = Vec::with_capacity(count);
        for i in 0..count {
            let level = i as f64 / (count.saturating_sub(1).max(1) as f64);
            patches.push(Patch::new(RGB { r: level, g: level, b: level }));
        }
        PatchSet { patches }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchStrategy {
    Grayscale(usize),
    OptimizedSubset { grayscale_count: usize, color_count: usize },
}
