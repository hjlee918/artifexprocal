use color_science::types::XYZ;
use hal::error::MeterError;
use hal::traits::Meter;

/// Wraps a Meter and applies a 3×3 correction matrix to every XYZ reading.
pub struct CorrectingMeter<'a> {
    inner: &'a mut dyn Meter,
    matrix: hal_meters::profiling::CorrectionMatrix,
}

impl<'a> CorrectingMeter<'a> {
    pub fn new(inner: &'a mut dyn Meter, matrix: hal_meters::profiling::CorrectionMatrix) -> Self {
        Self { inner, matrix }
    }
}

impl Meter for CorrectingMeter<'_> {
    fn connect(&mut self) -> Result<(), MeterError> {
        self.inner.connect()
    }

    fn disconnect(&mut self) {
        self.inner.disconnect()
    }

    fn read_xyz(&mut self, integration_time_ms: u32) -> Result<XYZ, MeterError> {
        let xyz = self.inner.read_xyz(integration_time_ms)?;
        Ok(self.matrix.apply(&xyz))
    }

    fn model(&self) -> &str {
        self.inner.model()
    }
}
