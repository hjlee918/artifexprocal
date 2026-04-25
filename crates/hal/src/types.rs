pub use color_science::types::RGB;

pub struct Lut1D {
    pub channels: [Vec<f64>; 3],
    pub size: usize,
}

pub struct Lut3D {
    pub data: Vec<RGB>,
    pub size: usize,
}

pub struct RGBGain {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

pub enum PictureMode {
    Standard,
    Cinema,
    Game,
    ExpertDark,
    ExpertBright,
    Custom(String),
}
