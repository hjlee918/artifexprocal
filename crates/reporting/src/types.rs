use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ReportTemplate {
    QuickSummary,
    Detailed,
    PrePostComparison,
}

impl fmt::Display for ReportTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportTemplate::QuickSummary => write!(f, "Quick Summary"),
            ReportTemplate::Detailed => write!(f, "Detailed"),
            ReportTemplate::PrePostComparison => write!(f, "Pre/Post Comparison"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ReportFormat {
    Html,
    Pdf,
}

impl fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportFormat::Html => write!(f, "HTML"),
            ReportFormat::Pdf => write!(f, "PDF"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("Template not supported: {0:?}")]
    UnsupportedTemplate(ReportTemplate),
    #[error("Missing comparison session for PrePostComparison")]
    MissingComparison,
    #[error("PDF generation failed: {0}")]
    PdfGenError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_template_display() {
        assert_eq!(ReportTemplate::QuickSummary.to_string(), "Quick Summary");
        assert_eq!(ReportTemplate::Detailed.to_string(), "Detailed");
        assert_eq!(ReportTemplate::PrePostComparison.to_string(), "Pre/Post Comparison");
    }

    #[test]
    fn test_report_format_display() {
        assert_eq!(ReportFormat::Html.to_string(), "HTML");
        assert_eq!(ReportFormat::Pdf.to_string(), "PDF");
    }

    #[test]
    fn test_report_error_display() {
        let err = ReportError::MissingComparison;
        assert_eq!(err.to_string(), "Missing comparison session for PrePostComparison");
    }
}
