use crate::error::{MeterError, DisplayError, PatternGenError};
use crate::traits::{Meter, DisplayController, PatternGenerator};
use crate::types::{Lut1D, Lut3D, RGBGain};
use color_science::types::{XYZ, RGB};

#[derive(Default)]
pub struct FakeMeter {
    connected: bool,
    preset_xyz: XYZ,
}

impl FakeMeter {
    pub fn with_preset(xyz: XYZ) -> Self {
        Self {
            connected: false,
            preset_xyz: xyz,
        }
    }
}

impl Meter for FakeMeter {
    fn connect(&mut self) -> Result<(), MeterError> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
    }

    fn read_xyz(&mut self, _integration_time_ms: u32) -> Result<XYZ, MeterError> {
        if !self.connected {
            return Err(MeterError::ConnectionFailed("Not connected".to_string()));
        }
        Ok(self.preset_xyz)
    }

    fn model(&self) -> &str {
        "FakeMeter"
    }
}

#[derive(Default)]
pub struct FakeDisplayController {
    connected: bool,
    pub picture_mode_calls: Vec<String>,
    pub uploaded_1d_luts: Vec<Lut1D>,
    pub uploaded_3d_luts: Vec<Lut3D>,
    pub white_balance_calls: Vec<RGBGain>,
}

impl DisplayController for FakeDisplayController {
    fn connect(&mut self) -> Result<(), DisplayError> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
    }

    fn model(&self) -> &str {
        "FakeDisplay"
    }

    fn set_picture_mode(&mut self, mode: &str) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        self.picture_mode_calls.push(mode.to_string());
        Ok(())
    }

    fn upload_1d_lut(&mut self, lut: &Lut1D) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        self.uploaded_1d_luts.push(Lut1D {
            channels: [lut.channels[0].clone(), lut.channels[1].clone(), lut.channels[2].clone()],
            size: lut.size,
        });
        Ok(())
    }

    fn upload_3d_lut(&mut self, lut: &Lut3D) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        self.uploaded_3d_luts.push(Lut3D {
            data: lut.data.clone(),
            size: lut.size,
        });
        Ok(())
    }

    fn set_white_balance(&mut self, gains: RGBGain) -> Result<(), DisplayError> {
        if !self.connected {
            return Err(DisplayError::ConnectionFailed("Not connected".to_string()));
        }
        self.white_balance_calls.push(RGBGain { r: gains.r, g: gains.g, b: gains.b });
        Ok(())
    }
}

#[derive(Default)]
pub struct FakePatternGenerator {
    connected: bool,
    pub patch_history: Vec<RGB>,
}

impl PatternGenerator for FakePatternGenerator {
    fn connect(&mut self) -> Result<(), PatternGenError> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
    }

    fn display_patch(&mut self, color: &RGB) -> Result<(), PatternGenError> {
        if !self.connected {
            return Err(PatternGenError::ConnectionFailed("Not connected".to_string()));
        }
        self.patch_history.push(RGB { r: color.r, g: color.g, b: color.b });
        Ok(())
    }
}
