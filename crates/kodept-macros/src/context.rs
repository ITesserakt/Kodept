use crate::error::report::{IntoSpannedReportMessage, Report};
use crate::error::report_collector::ReportCollector;
use kodept_ast::graph::stage::FullAccess;
use kodept_ast::graph::{AnyNodeD, GenericNodeId, SyntaxTree};
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_core::file_name::FileName;

pub type FileId = u16;

#[derive(Debug)]
pub struct FileDescriptor {
    pub name: FileName,
    pub id: FileId
}

#[derive(Debug)]
pub struct Context<'r> {
    pub ast: SyntaxTree<FullAccess>,
    pub rlt: RLTAccessor<'r>,
    pub reports: ReportCollector,
    pub current_file: FileDescriptor,
}

impl Context<'_> {
    pub fn describe(&self, node_id: GenericNodeId) -> AnyNodeD {
        self.ast
            .get(node_id)
            .expect("Cannot find node with given id")
            .describe()
    }
    
    pub fn report_and_fail<T>(&mut self, message: impl IntoSpannedReportMessage) -> Result<T, Report<FileId>> {
        Err(Report::from_message(self.current_file.id, message))
    }

    pub fn report(&mut self, message: impl IntoSpannedReportMessage) {
        self.reports.report(self.current_file.id, message)
    }
}
