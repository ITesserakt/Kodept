use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_core::Freeze;
use kodept_report::error::report::{IntoSpannedReportMessage, Report};
use kodept_report::error::report_collector::{ReportCollector, Reporter};
use kodept_report::{FileDescriptor, FileId};

#[derive(Debug)]
pub struct Context<'r> {
    pub ast: AST,
    pub rlt: SyntaxResolver,
    pub collector: &'r ReportCollector,
    pub current_file: Freeze<FileDescriptor>,
}

impl<'rlt> Context<'rlt> {
    pub fn report_and_fail<T>(
        &mut self,
        message: impl IntoSpannedReportMessage,
    ) -> Result<T, Report<FileId>> {
        Err(Report::from_message(self.current_file.id, message))
    }

    pub fn report(&self, message: impl IntoSpannedReportMessage) {
        self.collector.report(self.current_file.id, message)
    }
}
