use calibration_core::state::{CalibrationTier, TargetSpace, ToneCurve, WhitePoint};
use calibration_storage::query::{PatchReading, SessionDetail};
use color_science::delta_e::delta_e_2000;
use color_science::types::{RGB, WhitePoint as CsWhitePoint};

use crate::assets::REPORT_CSS;
use crate::svg::{cie_diagram_svg, de_bar_chart_svg, grayscale_tracker_svg};
use crate::types::{ReportError, ReportTemplate};

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn format_timestamp(ts: i64) -> String {
    let dt = chrono::DateTime::from_timestamp(ts, 0)
        .unwrap_or_else(chrono::Utc::now);
    dt.format("%Y-%m-%d %H:%M UTC").to_string()
}

fn format_target_space(ts: &TargetSpace) -> String {
    match ts {
        TargetSpace::Bt709 => "Rec.709".to_string(),
        TargetSpace::Bt2020 => "Rec.2020".to_string(),
        TargetSpace::DciP3 => "DCI-P3".to_string(),
        TargetSpace::Custom { .. } => "Custom".to_string(),
    }
}

fn format_tone_curve(tc: &ToneCurve) -> String {
    match tc {
        ToneCurve::Gamma(g) => format!("Gamma {:.2}", g),
        ToneCurve::Bt1886 => "BT.1886".to_string(),
        ToneCurve::Pq => "PQ (ST.2084)".to_string(),
        ToneCurve::Hlg => "HLG".to_string(),
        ToneCurve::Custom => "Custom".to_string(),
    }
}

fn format_white_point(wp: &WhitePoint) -> String {
    match wp {
        WhitePoint::D65 => "D65".to_string(),
        WhitePoint::D50 => "D50".to_string(),
        WhitePoint::Dci => "DCI".to_string(),
        WhitePoint::Custom(xyz) => format!("Custom ({:.4}, {:.4})", xyz.x, xyz.y),
    }
}

fn format_tier(tier: &CalibrationTier) -> String {
    match tier {
        CalibrationTier::GrayscaleOnly => "Grayscale Only".to_string(),
        CalibrationTier::GrayscalePlus3D => "Grayscale + 3D LUT".to_string(),
        CalibrationTier::Full3D => "Full 3D LUT".to_string(),
    }
}

fn format_de(v: f64) -> String {
    format!("{:.2}", v)
}

fn compute_de_for_reading(reading: &PatchReading) -> f64 {
    let target_rgb = RGB {
        r: reading.target_rgb.0,
        g: reading.target_rgb.1,
        b: reading.target_rgb.2,
    };
    let target_xyz = target_rgb.to_xyz_srgb();
    let target_lab = target_xyz.to_lab(CsWhitePoint::D65);
    let measured_lab = reading.measured_xyz.to_lab(CsWhitePoint::D65);
    delta_e_2000(&target_lab, &measured_lab)
}

fn html_page(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
{css}
</style>
</head>
<body>
{body}
</body>
</html>"#,
        title = escape_html(title),
        css = REPORT_CSS,
        body = body
    )
}

fn metric_card(label: &str, value: &str, unit: Option<&str>) -> String {
    let unit_html = unit.map_or_else(String::new, |u| {
        format!(r#"<span class="metric-unit">{}</span>"#, escape_html(u))
    });
    format!(
        r#"<div class="metric-card">
<div class="metric-label">{}</div>
<div class="metric-value">{} {}</div>
</div>"#,
        escape_html(label),
        escape_html(value),
        unit_html
    )
}

fn render_summary_metrics(detail: &SessionDetail) -> String {
    let summary = &detail.summary;
    let results = detail.results.as_ref();

    let mut cards = vec![
        metric_card(
            "Target Space",
            &format_target_space(&detail.config.target_space),
            None,
        ),
        metric_card(
            "Tone Curve",
            &format_tone_curve(&detail.config.tone_curve),
            None,
        ),
        metric_card(
            "White Point",
            &format_white_point(&detail.config.white_point),
            None,
        ),
        metric_card("Patch Count", &summary.patch_count.to_string(), None),
        metric_card("Tier", &format_tier(&detail.config.tier), None),
    ];

    if let Some(r) = results {
        if let Some(g) = r.gamma {
            cards.push(metric_card("Gamma", &format!("{:.2}", g), None));
        }
        if let Some(max_de) = r.max_de {
            cards.push(metric_card("Max ΔE", &format_de(max_de), None));
        }
        if let Some(avg_de) = r.avg_de {
            cards.push(metric_card("Avg ΔE", &format_de(avg_de), None));
        }
        if let Some(ref wb) = r.white_balance {
            cards.push(metric_card("White Balance", wb, None));
        }
    } else {
        if let Some(g) = summary.gamma {
            cards.push(metric_card("Gamma", &format!("{:.2}", g), None));
        }
        if let Some(max_de) = summary.max_de {
            cards.push(metric_card("Max ΔE", &format_de(max_de), None));
        }
        if let Some(avg_de) = summary.avg_de {
            cards.push(metric_card("Avg ΔE", &format_de(avg_de), None));
        }
    }

    format!(r#"<div class="metric-grid">{}</div>"#, cards.concat())
}

fn render_readings_table(detail: &SessionDetail) -> String {
    let mut rows = Vec::new();
    rows.push(
        r#"<tr><th>Patch</th><th>Target RGB</th><th>Measured XYZ</th><th>ΔE</th></tr>"#.to_string(),
    );

    let mut readings_with_de: Vec<_> = detail
        .readings
        .iter()
        .map(|r| {
            let de = compute_de_for_reading(r);
            (r, de)
        })
        .collect();

    readings_with_de.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (reading, de) in readings_with_de {
        let (tr, tg, tb) = reading.target_rgb;
        let xyz = &reading.measured_xyz;
        rows.push(format!(
            r#"<tr>
<td>{}</td>
<td>{:.3}, {:.3}, {:.3}</td>
<td>{:.2}, {:.2}, {:.2}</td>
<td>{}</td>
</tr>"#,
            reading.patch_index,
            tr, tg, tb,
            xyz.x, xyz.y, xyz.z,
            format_de(de)
        ));
    }

    format!(r#"<table>{}</table>"#, rows.concat())
}

pub fn render_quick_summary(detail: &SessionDetail) -> String {
    let summary = &detail.summary;
    let header = format!(
        r#"<div class="header">
<h1>{}</h1>
<div class="subtitle">{} | {}</div>
</div>"#,
        escape_html(&summary.name),
        format_timestamp(summary.created_at),
        escape_html(&summary.target_space)
    );

    let metrics = render_summary_metrics(detail);

    let charts = if !detail.readings.is_empty() {
        let cie = cie_diagram_svg(&detail.readings, 400, 300);
        let gray = grayscale_tracker_svg(&detail.readings, 400, 120);
        format!(
            r#"<div class="section">
<h2 class="section-title">CIE 1976 u'v' Diagram</h2>
<div class="chart-container">{}</div>
<h2 class="section-title">Grayscale Tracker</h2>
<div class="chart-container">{}</div>
</div>"#,
            cie, gray
        )
    } else {
        String::new()
    };

    let footer = r#"<div class="footer">Generated by ArtifexProCal</div>"#;

    let body = format!("{}{}{}{}", header, metrics, charts, footer);
    html_page(&format!("Quick Summary — {}", summary.name), &body)
}

pub fn render_detailed(detail: &SessionDetail) -> String {
    let summary = &detail.summary;
    let header = format!(
        r#"<div class="header">
<h1>Detailed Calibration Report</h1>
<div class="subtitle">{} | {} | {}</div>
</div>"#,
        escape_html(&summary.name),
        format_timestamp(summary.created_at),
        escape_html(&summary.target_space)
    );

    let metrics = render_summary_metrics(detail);

    let config_section = format!(
        r#"<div class="section">
<h2 class="section-title">Configuration</h2>
<table>
<tr><td>Session Name</td><td>{}</td></tr>
<tr><td>Target Space</td><td>{}</td></tr>
<tr><td>Tone Curve</td><td>{}</td></tr>
<tr><td>White Point</td><td>{}</td></tr>
<tr><td>Patch Count</td><td>{}</td></tr>
<tr><td>Reads Per Patch</td><td>{}</td></tr>
<tr><td>Settle Time</td><td>{} ms</td></tr>
<tr><td>Tier</td><td>{}</td></tr>
</table>
</div>"#,
        escape_html(&detail.config.name),
        format_target_space(&detail.config.target_space),
        format_tone_curve(&detail.config.tone_curve),
        format_white_point(&detail.config.white_point),
        detail.config.patch_count,
        detail.config.reads_per_patch,
        detail.config.settle_time_ms,
        format_tier(&detail.config.tier)
    );

    let charts = if !detail.readings.is_empty() {
        let cie = cie_diagram_svg(&detail.readings, 600, 450);
        let gray = grayscale_tracker_svg(&detail.readings, 600, 200);
        let de_bars = de_bar_chart_svg(&detail.readings, &[], 600, 200);
        format!(
            r#"<div class="section">
<h2 class="section-title">CIE 1976 u'v' Diagram</h2>
<div class="chart-container">{}</div>
<h2 class="section-title">Grayscale Tracker</h2>
<div class="chart-container">{}</div>
<h2 class="section-title">dE Bar Chart</h2>
<div class="chart-container">{}</div>
</div>"#,
            cie, gray, de_bars
        )
    } else {
        String::new()
    };

    let readings_section = if !detail.readings.is_empty() {
        format!(
            r#"<div class="section">
<h2 class="section-title">Patch Readings (sorted by ΔE)</h2>
{}
</div>"#,
            render_readings_table(detail)
        )
    } else {
        String::new()
    };

    let lut_section = if let Some(ref results) = detail.results {
        let mut rows = Vec::new();
        if let Some(size) = results.lut_1d_size {
            rows.push(format!(
                r#"<tr><td>1D LUT</td><td>{} entries</td></tr>"#,
                size
            ));
        }
        if let Some(size) = results.lut_3d_size {
            rows.push(format!(
                r#"<tr><td>3D LUT</td><td>{} points</td></tr>"#,
                size
            ));
        }
        if !rows.is_empty() {
            format!(
                r#"<div class="section">
<h2 class="section-title">LUT Metadata</h2>
<table>{}</table>
</div>"#,
                rows.concat()
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let footer = r#"<div class="footer">Generated by ArtifexProCal — Detailed Report</div>"#;

    let body = format!("{}{}{}{}{}{}{}", header, metrics, config_section, charts, readings_section, lut_section, footer);
    html_page(&format!("Detailed Report — {}", summary.name), &body)
}

pub fn render_pre_post_comparison(before: &SessionDetail, after: &SessionDetail) -> String {
    let before_summary = &before.summary;
    let after_summary = &after.summary;

    let header = format!(
        r#"<div class="header">
<h1>Pre/Post Calibration Comparison</h1>
<div class="subtitle">Before: {} | After: {}</div>
</div>"#,
        escape_html(&before_summary.name),
        escape_html(&after_summary.name)
    );

    let comparison_grid = {
        let b_results = before.results.as_ref();
        let a_results = after.results.as_ref();

        let mut before_cards = Vec::new();
        let mut after_cards = Vec::new();
        let mut delta_rows = Vec::new();

        before_cards.push(metric_card("Target", &format_target_space(&before.config.target_space), None));
        after_cards.push(metric_card("Target", &format_target_space(&after.config.target_space), None));
        delta_rows.push(r#"<tr><td>Target</td><td>—</td></tr>"#.to_string());

        if let (Some(br), Some(ar)) = (b_results, a_results) {
            if let (Some(bg), Some(ag)) = (br.gamma, ar.gamma) {
                before_cards.push(metric_card("Gamma", &format!("{:.2}", bg), None));
                after_cards.push(metric_card("Gamma", &format!("{:.2}", ag), None));
                let delta = ag - bg;
                let cls = if delta.abs() < 0.05 { "delta-positive" } else { "delta-negative" };
                delta_rows.push(format!(
                    r#"<tr><td>Gamma</td><td class="{}">{:+.2}</td></tr>"#,
                    cls, delta
                ));
            }
            if let (Some(bm), Some(am)) = (br.max_de, ar.max_de) {
                before_cards.push(metric_card("Max ΔE", &format_de(bm), None));
                after_cards.push(metric_card("Max ΔE", &format_de(am), None));
                let delta = am - bm;
                let cls = if delta < 0.0 { "delta-positive" } else { "delta-negative" };
                delta_rows.push(format!(
                    r#"<tr><td>Max ΔE</td><td class="{}">{:+.2}</td></tr>"#,
                    cls, delta
                ));
            }
            if let (Some(ba), Some(aa)) = (br.avg_de, ar.avg_de) {
                before_cards.push(metric_card("Avg ΔE", &format_de(ba), None));
                after_cards.push(metric_card("Avg ΔE", &format_de(aa), None));
                let delta = aa - ba;
                let cls = if delta < 0.0 { "delta-positive" } else { "delta-negative" };
                delta_rows.push(format!(
                    r#"<tr><td>Avg ΔE</td><td class="{}">{:+.2}</td></tr>"#,
                    cls, delta
                ));
            }
        } else {
            if let (Some(bm), Some(am)) = (before_summary.max_de, after_summary.max_de) {
                before_cards.push(metric_card("Max ΔE", &format_de(bm), None));
                after_cards.push(metric_card("Max ΔE", &format_de(am), None));
                let delta = am - bm;
                let cls = if delta < 0.0 { "delta-positive" } else { "delta-negative" };
                delta_rows.push(format!(
                    r#"<tr><td>Max ΔE</td><td class="{}">{:+.2}</td></tr>"#,
                    cls, delta
                ));
            }
            if let (Some(ba), Some(aa)) = (before_summary.avg_de, after_summary.avg_de) {
                before_cards.push(metric_card("Avg ΔE", &format_de(ba), None));
                after_cards.push(metric_card("Avg ΔE", &format_de(aa), None));
                let delta = aa - ba;
                let cls = if delta < 0.0 { "delta-positive" } else { "delta-negative" };
                delta_rows.push(format!(
                    r#"<tr><td>Avg ΔE</td><td class="{}">{:+.2}</td></tr>"#,
                    cls, delta
                ));
            }
        }

        format!(
            r#"<div class="comparison-grid">
<div class="comparison-card">
<h3>Before</h3>
<div class="metric-grid">{}</div>
</div>
<div class="comparison-card">
<h3>After</h3>
<div class="metric-grid">{}</div>
</div>
</div>
<div class="section">
<h2 class="section-title">Delta Summary</h2>
<table>{}</table>
</div>"#,
            before_cards.concat(),
            after_cards.concat(),
            delta_rows.concat()
        )
    };

    let charts = if !before.readings.is_empty() || !after.readings.is_empty() {
        let before_cie = if !before.readings.is_empty() {
            format!(
                r#"<div><h4>Before</h4>{}</div>"#,
                cie_diagram_svg(&before.readings, 360, 270)
            )
        } else {
            String::new()
        };
        let after_cie = if !after.readings.is_empty() {
            format!(
                r#"<div><h4>After</h4>{}</div>"#,
                cie_diagram_svg(&after.readings, 360, 270)
            )
        } else {
            String::new()
        };
        format!(
            r#"<div class="section">
<h2 class="section-title">CIE Diagrams</h2>
<div class="comparison-grid">
{}{}
</div>
</div>"#,
            before_cie, after_cie
        )
    } else {
        String::new()
    };

    let footer = r#"<div class="footer">Generated by ArtifexProCal — Comparison Report</div>"#;

    let body = format!("{}{}{}{}", header, comparison_grid, charts, footer);
    html_page("Pre/Post Calibration Comparison", &body)
}

pub fn render_html(
    template: ReportTemplate,
    detail: &SessionDetail,
    compare: Option<&SessionDetail>,
) -> Result<String, ReportError> {
    match template {
        ReportTemplate::QuickSummary => Ok(render_quick_summary(detail)),
        ReportTemplate::Detailed => Ok(render_detailed(detail)),
        ReportTemplate::PrePostComparison => {
            let compare = compare.ok_or(ReportError::MissingComparison)?;
            Ok(render_pre_post_comparison(detail, compare))
        }
    }
}
