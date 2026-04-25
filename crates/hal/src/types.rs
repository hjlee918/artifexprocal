pub struct Lut1D {
    pub channels: usize,
    pub entries: usize,
    pub data: Vec<f32>,
}

pub struct Lut3D {
    pub size: usize,
    pub data: Vec<f32>,
}

pub struct RGBGain {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}
