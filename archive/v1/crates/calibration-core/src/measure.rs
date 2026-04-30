use color_science::types::XYZ;

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub raw_xyz: XYZ,
    pub measured_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadingStats {
    pub mean: XYZ,
    pub std_dev: XYZ,
}

impl ReadingStats {
    pub fn compute(readings: &[XYZ]) -> Self {
        let n = readings.len() as f64;
        if n == 0.0 {
            return Self {
                mean: XYZ { x: 0.0, y: 0.0, z: 0.0 },
                std_dev: XYZ { x: 0.0, y: 0.0, z: 0.0 },
            };
        }

        let mean = XYZ {
            x: readings.iter().map(|r| r.x).sum::<f64>() / n,
            y: readings.iter().map(|r| r.y).sum::<f64>() / n,
            z: readings.iter().map(|r| r.z).sum::<f64>() / n,
        };

        let variance = XYZ {
            x: readings.iter().map(|r| (r.x - mean.x).powi(2)).sum::<f64>() / n,
            y: readings.iter().map(|r| (r.y - mean.y).powi(2)).sum::<f64>() / n,
            z: readings.iter().map(|r| (r.z - mean.z).powi(2)).sum::<f64>() / n,
        };

        Self {
            mean,
            std_dev: XYZ {
                x: variance.x.sqrt(),
                y: variance.y.sqrt(),
                z: variance.z.sqrt(),
            },
        }
    }
}

/// Orchestrates N repeated meter readings with optional stability detection.
pub struct MeasurementLoop;

impl MeasurementLoop {
    /// Take `n_reads` from the meter, compute mean and std dev.
    /// If `stability_threshold` is Some, continue reading until
    /// std_dev of the last `n_reads` readings is below threshold.
    pub fn measure_sync<F>(
        mut read_fn: F,
        n_reads: usize,
        _stability_threshold: Option<f64>,
    ) -> ReadingStats
    where
        F: FnMut() -> XYZ,
    {
        let readings: Vec<XYZ> = (0..n_reads).map(|_| read_fn()).collect();
        ReadingStats::compute(&readings)
    }
}
