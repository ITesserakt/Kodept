use crate::context::FileId;
use crate::error::report::{IntoSpannedReportMessage, Report};

#[derive(Default, Debug)]
pub struct ReportCollector {
    reports: Vec<Report<FileId>>,
    has_errors: bool,
}

impl ReportCollector {
    #[must_use]
    pub const fn new() -> Self {
        ReportCollector {
            reports: vec![],
            has_errors: false,
        }
    }

    pub fn push_report(&mut self, report: Report<FileId>) {
        self.has_errors |= report.is_error();
        self.reports.push(report)
    }

    pub fn report(&mut self, file_id: FileId, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(file_id, message);
        self.push_report(report)
    }

    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.has_errors
    }

    pub fn has_reports(&self) -> bool {
        !self.reports.is_empty()
    }

    #[must_use]
    pub fn into_collected_reports(self) -> Vec<Report<FileId>> {
        self.reports
    }

    pub fn as_collected_reports(&self) -> &[Report<FileId>] {
        self.reports.as_slice()
    }
}
