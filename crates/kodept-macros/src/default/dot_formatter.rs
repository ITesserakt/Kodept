use crate::context::{Context, SyntaxProvider};
use crate::error::report::ReportMessage;
use crate::visit_guard::VisitGuard;
use crate::Macro;
use codespan_reporting::diagnostic::Severity;
use kodept_ast::graph::ChangeSet;
use kodept_ast::utils::Execution;
use kodept_ast::utils::Execution::{Completed, Skipped};
use kodept_ast::FileDecl;
use std::fmt::Write;

pub struct ASTDotFormatter<W> {
    output: W,
}

#[derive(Debug)]
pub struct Error(std::fmt::Error);

impl From<Error> for ReportMessage {
    fn from(value: Error) -> Self {
        Self::new(Severity::Error, "IO001", value.0.to_string())
    }
}

impl<C: SyntaxProvider, W: Write> Macro<C> for ASTDotFormatter<W> {
    type Error = Error;
    type Node = FileDecl;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut impl Context<C>,
    ) -> Execution<Self::Error, ChangeSet> {
        if guard.allow_last().is_none() {
            return Skipped;
        }
        
        write!(&mut self.output, "{}", ctx.export_dot(&[])).map_err(Error)?;
        Completed(ChangeSet::new())
    }
}
