//! module-meter — MeterModule CalibrationModule implementation.

pub struct MeterModule;

impl app_core::CalibrationModule for MeterModule {
    fn module_id(&self) -> &'static str {
        "meter"
    }
}
