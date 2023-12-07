use crate::error::report::Report;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Default, Debug)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct ReportCollector {
    reports: Vec<Report>,
    has_errors: bool,
}

impl ReportCollector {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn report(&mut self, report: Report) {
        self.has_errors |= report.is_error();
        self.reports.push(report);
    }

    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.has_errors
    }

    pub fn has_reports(&self) -> bool {
        !self.reports.is_empty()
    }

    #[must_use]
    pub fn into_collected_reports(self) -> Vec<Report> {
        self.reports
    }

    pub fn as_collected_reports(&self) -> &[Report] {
        self.reports.as_slice()
    }
}
