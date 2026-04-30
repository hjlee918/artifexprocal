use calibration_storage::query::SessionDetail;

use crate::pdf::generate_pdf;
use crate::template::render_html;
use crate::types::{ReportError, ReportFormat, ReportTemplate};

pub struct ReportEngine;

impl ReportEngine {
    pub fn generate(
        template: ReportTemplate,
        format: ReportFormat,
        detail: &SessionDetail,
        compare_detail: Option<&SessionDetail>,
    ) -> Result<Vec<u8>, ReportError> {
        match format {
            ReportFormat::Html => {
                let html = render_html(template, detail, compare_detail)?;
                Ok(html.into_bytes())
            }
            ReportFormat::Pdf => generate_pdf(template, detail, compare_detail),
        }
    }
}
