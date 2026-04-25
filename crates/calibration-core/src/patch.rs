use color_science::types::RGB;

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub target_rgb: RGB,
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
    pub fn new(count: usize) -> PatchSet {
        let mut patches = Vec::with_capacity(count);
        for i in 0..count {
            let level = i as f64 / (count.saturating_sub(1).max(1) as f64);
            patches.push(Patch {
                target_rgb: RGB { r: level, g: level, b: level },
            });
        }
        PatchSet { patches }
    }
}
