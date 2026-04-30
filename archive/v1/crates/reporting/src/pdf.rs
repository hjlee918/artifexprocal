use calibration_storage::query::SessionDetail;
use printpdf::*;
use std::io::BufWriter;

use crate::types::{ReportError, ReportTemplate};

#[allow(unused_assignments)]
pub fn generate_pdf(
    _template: ReportTemplate,
    detail: &SessionDetail,
    compare: Option<&SessionDetail>,
) -> Result<Vec<u8>, ReportError> {
    let (doc, page1, layer1) = PdfDocument::new(
        "Calibration Report",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| ReportError::PdfGenError(format!("Font error: {e}")))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| ReportError::PdfGenError(format!("Font error: {e}")))?;

    let mut y = Mm(280.0);

    // Title
    current_layer.use_text(
        &detail.summary.name,
        18.0,
        Mm(15.0),
        y,
        &font_bold,
    );
    y = Mm(y.0 - 10.0);

    current_layer.use_text(
        format!("Generated: {}", chrono::Utc::now().format("%Y-%m-%d")),
        10.0,
        Mm(15.0),
        y,
        &font,
    );
    y = Mm(y.0 - 15.0);

    // Summary section
    current_layer.use_text("Summary", 14.0, Mm(15.0), y, &font_bold);
    y = Mm(y.0 - 8.0);

    current_layer.use_text(
        format!("Target Space: {}", detail.summary.target_space),
        10.0,
        Mm(15.0),
        y,
        &font,
    );
    y = Mm(y.0 - 5.0);

    current_layer.use_text(
        format!("Patch Count: {}", detail.summary.patch_count),
        10.0,
        Mm(15.0),
        y,
        &font,
    );
    y = Mm(y.0 - 5.0);

    if let Some(ref results) = detail.results {
        if let Some(g) = results.gamma {
            current_layer.use_text(
                format!("Gamma: {:.2}", g),
                10.0,
                Mm(15.0),
                y,
                &font,
            );
            y = Mm(y.0 - 5.0);
        }
        if let Some(max_de) = results.max_de {
            current_layer.use_text(
                format!("Max dE: {:.2}", max_de),
                10.0,
                Mm(15.0),
                y,
                &font,
            );
            y = Mm(y.0 - 5.0);
        }
        if let Some(avg_de) = results.avg_de {
            current_layer.use_text(
                format!("Avg dE: {:.2}", avg_de),
                10.0,
                Mm(15.0),
                y,
                &font,
            );
            y = Mm(y.0 - 5.0);
        }
        if let Some(ref wb) = results.white_balance {
            current_layer.use_text(
                format!("White Balance: {wb}"),
                10.0,
                Mm(15.0),
                y,
                &font,
            );
            y = Mm(y.0 - 5.0);
        }
    } else {
        if let Some(g) = detail.summary.gamma {
            current_layer.use_text(
                format!("Gamma: {:.2}", g),
                10.0,
                Mm(15.0),
                y,
                &font,
            );
            y = Mm(y.0 - 5.0);
        }
        if let Some(max_de) = detail.summary.max_de {
            current_layer.use_text(
                format!("Max dE: {:.2}", max_de),
                10.0,
                Mm(15.0),
                y,
                &font,
            );
            y = Mm(y.0 - 5.0);
        }
        if let Some(avg_de) = detail.summary.avg_de {
            current_layer.use_text(
                format!("Avg dE: {:.2}", avg_de),
                10.0,
                Mm(15.0),
                y,
                &font,
            );
            y = Mm(y.0 - 5.0);
        }
    }

    // If comparison, add second page
    if let Some(compare_detail) = compare {
        let (page2, layer2) = doc.add_page(Mm(210.0), Mm(297.0), "Comparison");
        let comp_layer = doc.get_page(page2).get_layer(layer2);
        let mut y2 = Mm(280.0);

        comp_layer.use_text("Comparison Report", 18.0, Mm(15.0), y2, &font_bold);
        y2 = Mm(y2.0 - 10.0);
        comp_layer.use_text(
            format!("Before: {}", compare_detail.summary.name),
            10.0,
            Mm(15.0),
            y2,
            &font,
        );
        y2 = Mm(y2.0 - 5.0);
        comp_layer.use_text(
            format!("After: {}", detail.summary.name),
            10.0,
            Mm(15.0),
            y2,
            &font,
        );
        y2 = Mm(y2.0 - 10.0);

        comp_layer.use_text("Summary Metrics", 14.0, Mm(15.0), y2, &font_bold);
        y2 = Mm(y2.0 - 8.0);

        let before_results = compare_detail.results.as_ref();
        let after_results = detail.results.as_ref();

        if let (Some(br), Some(ar)) = (before_results, after_results) {
            if let (Some(bg), Some(ag)) = (br.gamma, ar.gamma) {
                comp_layer.use_text(
                    format!("Gamma: {:.2} -> {:.2} ({:+.2})", bg, ag, ag - bg),
                    10.0,
                    Mm(15.0),
                    y2,
                    &font,
                );
                y2 = Mm(y2.0 - 5.0);
            }
            if let (Some(bm), Some(am)) = (br.max_de, ar.max_de) {
                comp_layer.use_text(
                    format!("Max dE: {:.2} -> {:.2} ({:+.2})", bm, am, am - bm),
                    10.0,
                    Mm(15.0),
                    y2,
                    &font,
                );
                y2 = Mm(y2.0 - 5.0);
            }
            if let (Some(ba), Some(aa)) = (br.avg_de, ar.avg_de) {
                comp_layer.use_text(
                    format!("Avg dE: {:.2} -> {:.2} ({:+.2})", ba, aa, aa - ba),
                    10.0,
                    Mm(15.0),
                    y2,
                    &font,
                );
                y2 = Mm(y2.0 - 5.0);
            }
        }
    }

    let mut buf = BufWriter::new(Vec::new());
    doc.save(&mut buf)
        .map_err(|e| ReportError::PdfGenError(format!("PDF save error: {e}")))?;

    buf.into_inner()
        .map_err(|e| ReportError::PdfGenError(format!("Buffer error: {e}")))
}
