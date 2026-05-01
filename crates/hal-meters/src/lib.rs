//! hal-meters — Meter driver implementations.

pub struct FakeMeter;

impl hal::Meter for FakeMeter {
    fn read_xyz(&mut self) -> Result<( f64, f64, f64), hal::MeterError> {
        Ok((95.047, 100.0, 108.883))
    }
}
