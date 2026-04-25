use crate::error::DisplayError;
use crate::types::{Lut1D, Lut3D, RGBGain};
use color_science::types::XYZ;

pub trait DisplayController {
    fn connect(&mut self) -> Result<(), DisplayError>;
    fn disconnect(&mut self);
    fn set_picture_mode(&mut self, mode: &str) -> Result<(), DisplayError>;
    fn upload_1d_lut(&mut self, lut: &Lut1D) -> Result<(), DisplayError>;
    fn upload_3d_lut(&mut self, lut: &Lut3D) -> Result<(), DisplayError>;
    fn set_white_balance(&mut self, gains: RGBGain) -> Result<(), DisplayError>;
}

use crate::error::MeterError;

pub trait Meter {
    fn connect(&mut self) -> Result<(), MeterError>;
    fn disconnect(&mut self);
    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError>;
    fn model(&self) -> &str;
}
