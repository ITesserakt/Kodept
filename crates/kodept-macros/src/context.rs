use crate::error::report::{Report, ReportMessage};
use crate::unrecoverable_error::UnrecoverableError;
use kodept_ast::graph::stage::FullAccess;
use kodept_ast::graph::{AnyNodeD, GenericNodeId, SyntaxTree};
use kodept_core::code_point::CodePoint;
use kodept_core::file_relative::CodePath;
use std::ops::DerefMut;

pub trait Context<Capability = ()>: DerefMut<Target = Capability> {
    fn enrich<R>(self, f: impl FnOnce(Capability) -> R) -> impl Context<R>;
}

pub trait SyntaxProvider: DerefMut<Target = SyntaxTree<FullAccess>> {
    #[inline(always)]
    fn describe(&self, id: GenericNodeId) -> AnyNodeD {
        let node = self.deref().get(id).unwrap();
        node.describe()
    }
}

pub trait FileProvider {
    fn path(&self) -> CodePath;
}

pub trait Reporter: FileProvider {
    #[deprecated]
    fn report_and_fail<R: Into<ReportMessage>, T>(
        &self,
        at: Vec<CodePoint>,
        message: R,
    ) -> Result<T, UnrecoverableError> {
        Err(Report::new(&self.path(), at, message).into())
    }

    #[deprecated]
    fn add_report<R: Into<ReportMessage>>(&mut self, at: Vec<CodePoint>, message: R) {
        self.report(Report::new(&self.path(), at, message))
    }

    fn report(&mut self, report: Report);
}
