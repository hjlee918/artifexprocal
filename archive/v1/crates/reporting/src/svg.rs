use calibration_storage::query::PatchReading;
use color_science::diagrams::{spectral_locus_1976, xy_to_uprime_vprime};

/// Generate a CIE 1976 u'v' diagram SVG with spectral locus and measured points.
pub fn cie_diagram_svg(readings: &[PatchReading], width: u32, height: u32) -> String {
    let margin = 40;
    let plot_w = width - 2 * margin;
    let plot_h = height - 2 * margin;

    // CIE 1976 u'v' bounds
    let u_min = 0.0;
    let u_max = 0.64;
    let v_min = 0.0;
    let v_max = 0.60;

    let scale_x = plot_w as f64 / (u_max - u_min);
    let scale_y = plot_h as f64 / (v_max - v_min);

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
<rect width="100%" height="100%" fill="#fafafa"/>
"##,
        width, height, width, height
    );

    // Draw spectral locus
    let locus = spectral_locus_1976();
    let locus_path: Vec<String> = locus
        .iter()
        .map(|(x, y)| {
            let (u, v) = xy_to_uprime_vprime(*x, *y);
            let px = margin + ((u - u_min) * scale_x) as u32;
            let py = height - margin - ((v - v_min) * scale_y) as u32;
            format!("{},{} ", px, py)
        })
        .collect();

    svg.push_str(&format!(
        r##"<polyline points="{}" fill="none" stroke="#3b82f6" stroke-width="1.5"/>
"##,
        locus_path.concat()
    ));

    // Draw measured points
    for reading in readings {
        let xyz = &reading.measured_xyz;
        let sum = xyz.x + xyz.y + xyz.z;
        if sum > 0.0 {
            let x = xyz.x / sum;
            let y = xyz.y / sum;
            let (u, v) = xy_to_uprime_vprime(x, y);
            let px = margin + ((u - u_min) * scale_x) as u32;
            let py = height - margin - ((v - v_min) * scale_y) as u32;
            svg.push_str(&format!(
                r##"<circle cx="{}" cy="{}" r="3" fill="#ef4444" opacity="0.7"/>
"##,
                px, py
            ));
        }
    }

    // Axis labels
    svg.push_str(&format!(
        r##"<text x="{}" y="{}" font-size="10" fill="#666" text-anchor="middle">u'</text>
<text x="{}" y="{}" font-size="10" fill="#666" text-anchor="middle" transform="rotate(-90, {}, {})">v'</text>
"##,
        margin + plot_w / 2,
        height - 10,
        15,
        margin + plot_h / 2,
        15,
        margin + plot_h / 2
    ));

    svg.push_str("</svg>");
    svg
}

/// Generate a grayscale tracker SVG showing RGB balance.
pub fn grayscale_tracker_svg(readings: &[PatchReading], width: u32, height: u32) -> String {
    let margin = 40;
    let bar_w = if readings.is_empty() {
        10
    } else {
        ((width - 2 * margin) / readings.len().max(1) as u32).saturating_sub(4).max(2)
    };
    let max_y = 120.0; // percentage

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
<rect width="100%" height="100%" fill="#fafafa"/>
"##,
        width, height, width, height
    );

    // Reference line at 100%
    let y100 = margin + ((height - 2 * margin) as f64 * (1.0 - 100.0 / max_y)) as u32;
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ccc" stroke-dasharray="4"/>
"##,
        margin,
        y100,
        width - margin,
        y100
    ));

    for (i, reading) in readings.iter().enumerate() {
        let x = margin + i as u32 * (bar_w + 4);
        let total = reading.target_rgb.0 + reading.target_rgb.1 + reading.target_rgb.2;
        if total > 0.0 {
            let r_pct = reading.target_rgb.0 / total * 100.0;
            let g_pct = reading.target_rgb.1 / total * 100.0;
            let b_pct = reading.target_rgb.2 / total * 100.0;

            let bar_h = height - 2 * margin;
            let y_base = height - margin;

            let r_h = (r_pct / max_y * bar_h as f64) as u32;
            let g_h = (g_pct / max_y * bar_h as f64) as u32;
            let b_h = (b_pct / max_y * bar_h as f64) as u32;

            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#ef4444" opacity="0.7"/>
<rect x="{}" y="{}" width="{}" height="{}" fill="#22c55e" opacity="0.7"/>
<rect x="{}" y="{}" width="{}" height="{}" fill="#3b82f6" opacity="0.7"/>
"##,
                x,
                y_base - r_h,
                bar_w / 3,
                r_h,
                x + bar_w / 3,
                y_base - g_h,
                bar_w / 3,
                g_h,
                x + 2 * bar_w / 3,
                y_base - b_h,
                bar_w / 3,
                b_h,
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

/// Generate a dE bar chart SVG.
pub fn de_bar_chart_svg(
    readings: &[PatchReading],
    _reference_xyz: &[(f64, f64, f64)],
    width: u32,
    height: u32,
) -> String {
    let margin = 40;
    let max_de = 5.0f64; // clamp scale

    let bar_w = if readings.is_empty() {
        10
    } else {
        ((width - 2 * margin) / readings.len().max(1) as u32).saturating_sub(2).max(2)
    };

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
<rect width="100%" height="100%" fill="#fafafa"/>
"##,
        width, height, width, height
    );

    // Reference line at dE=1.0
    let y1 = margin + ((height - 2 * margin) as f64 * (1.0 - 1.0 / max_de)) as u32;
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#22c55e" stroke-dasharray="4"/>
<text x="{}" y="{}" font-size="9" fill="#22c55e">dE=1.0</text>
"##,
        margin,
        y1,
        width - margin,
        y1,
        width - margin - 40,
        y1 - 4
    ));

    for (i, _reading) in readings.iter().enumerate() {
        let x = margin + i as u32 * (bar_w + 2);
        // For now, use a placeholder dE based on patch index
        let de = ((i as f64 + 1.0) / readings.len().max(1) as f64 * max_de).min(max_de);
        let bar_h = (de / max_de * (height - 2 * margin) as f64) as u32;
        let y = height - margin - bar_h;

        let color = if de < 1.0 {
            "#22c55e"
        } else if de < 3.0 {
            "#f59e0b"
        } else {
            "#ef4444"
        };

        svg.push_str(&format!(
            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.8"/>
"##,
            x, y, bar_w, bar_h, color
        ));
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_science::types::XYZ;

    fn make_reading(idx: usize, r: f64, g: f64, b: f64) -> PatchReading {
        PatchReading {
            patch_index: idx,
            target_rgb: (r, g, b),
            measured_xyz: XYZ { x: r * 95.0, y: g * 100.0, z: b * 108.0 },
            reading_index: 0,
            measurement_type: "cal".to_string(),
        }
    }

    #[test]
    fn test_cie_diagram_svg_contains_elements() {
        let readings = vec![make_reading(0, 1.0, 0.0, 0.0), make_reading(1, 0.0, 1.0, 0.0)];
        let svg = cie_diagram_svg(&readings, 400, 300);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("polyline"));
        assert!(svg.contains("circle"));
    }

    #[test]
    fn test_grayscale_tracker_svg_contains_elements() {
        let readings = vec![make_reading(0, 1.0, 1.0, 1.0), make_reading(1, 0.5, 0.5, 0.5)];
        let svg = grayscale_tracker_svg(&readings, 400, 300);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_de_bar_chart_svg_contains_elements() {
        let readings = vec![make_reading(0, 1.0, 0.0, 0.0), make_reading(1, 0.5, 0.5, 0.5)];
        let svg = de_bar_chart_svg(&readings, &[], 400, 300);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("dE=1.0"));
    }
}
