use crate::context::Context;
use crate::error::report::{ReportMessage, Severity};
use crate::execution::Execution;
use crate::execution::Execution::{Completed, Skipped};
use crate::visit_guard::VisitGuard;
use crate::Macro;
use kodept_ast::graph::ChangeSet;
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

impl<W: Write> Macro for ASTDotFormatter<W> {
    type Error = Error;
    type Node = FileDecl;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Execution<Self::Error, ChangeSet> {
        if guard.allow_last().is_none() {
            return Skipped;
        }
        
        write!(&mut self.output, "{}", ctx.ast.export_dot(&[])).map_err(Error)?;
        Completed(ChangeSet::new())
    }
}
