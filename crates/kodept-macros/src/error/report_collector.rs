use crate::context::FileId;
use crate::error::report::{IntoSpannedReportMessage, Report};
use append_only_vec::AppendOnlyVec;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default, Debug)]
pub struct ReportCollector<F = FileId> {
    reports: AppendOnlyVec<Report<F>>,
    has_errors: AtomicBool,
}

pub trait Reporter<F> {
    fn report(self, file_id: F, message: impl IntoSpannedReportMessage);
}

impl<F> ReportCollector<F> {
    #[must_use]
    pub const fn new() -> Self {
        ReportCollector {
            reports: AppendOnlyVec::new(),
            has_errors: AtomicBool::new(false),
        }
    }

    pub fn push_report(&self, report: Report<F>) {
        self.has_errors
            .fetch_or(report.is_error(), Ordering::AcqRel);
        self.reports.push(report);
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.has_errors.load(Ordering::Acquire)
    }

    pub fn has_reports(&self) -> bool {
        self.reports.len() != 0
    }

    #[must_use]
    pub fn into_collected_reports(self) -> Vec<Report<F>> {
        self.reports.into_vec()
    }
}

impl<F> Reporter<F> for &ReportCollector<F>
where
    F: Clone,
{
    fn report(self, file_id: F, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(file_id, message);
        self.has_errors
            .fetch_or(report.is_error(), Ordering::AcqRel);
        self.reports.push(report);
    }
}

impl<F> Reporter<F> for &mut ReportCollector<F>
where
    F: Clone,
{
    fn report(self, file_id: F, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(file_id, message);
        self.has_errors
            .fetch_or(report.is_error(), Ordering::AcqRel);
        self.reports.push_mut(report);
    }
}
