use calibration_autocal::greyscale::*;
use calibration_core::state::{TargetSpace, WhitePoint};
use color_science::types::{XYZ, RGB};

#[test]
fn test_analyze_perfect_greyscale() {
    // Perfect D65 greyscale: all patches neutral, gamma 2.2
    let readings: Vec<(RGB, XYZ)> = (0..=20)
        .map(|i| {
            let level = i as f64 / 20.0;
            let y = level.powf(2.2) * 100.0;
            (
                RGB { r: level, g: level, b: level },
                XYZ { x: y * 0.3127 / 0.3290, y, z: y * (1.0 - 0.3127 - 0.3290) / 0.3290 },
            )
        })
        .collect();

    let target = TargetSpace::Bt709;
    let white_point = WhitePoint::D65;
    let result = GreyscaleAnalyzer::analyze(&readings, &target, &white_point).unwrap();

    // Gamma estimate should be close to 2.2
    assert!((result.gamma - 2.2).abs() < 0.05, "gamma estimate should be close to 2.2, got {}", result.gamma);

    // For perfect input, correction factors should be near 1.0
    let last_corr = result.per_channel_corrections[0].last().unwrap();
    assert!((last_corr - 1.0).abs() < 0.01, "white correction should be ~1.0, got {}", last_corr);
}
