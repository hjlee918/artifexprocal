use calibration_core::measure::*;
use color_science::types::XYZ;

#[test]
fn test_reading_stats_mean() {
    let readings = vec![
        XYZ { x: 10.0, y: 20.0, z: 30.0 },
        XYZ { x: 12.0, y: 22.0, z: 32.0 },
        XYZ { x: 14.0, y: 24.0, z: 34.0 },
    ];
    let stats = ReadingStats::compute(&readings);
    assert_eq!(stats.mean.x, 12.0);
    assert_eq!(stats.mean.y, 22.0);
    assert_eq!(stats.mean.z, 32.0);
}

#[test]
fn test_reading_stats_std_dev() {
    let readings = vec![
        XYZ { x: 10.0, y: 20.0, z: 30.0 },
        XYZ { x: 12.0, y: 22.0, z: 32.0 },
        XYZ { x: 14.0, y: 24.0, z: 34.0 },
    ];
    let stats = ReadingStats::compute(&readings);
    // std dev of [10, 12, 14] = sqrt(((4+0+4)/3)) = sqrt(8/3) ≈ 1.633
    assert!((stats.std_dev.x - 1.632993).abs() < 0.001);
}
