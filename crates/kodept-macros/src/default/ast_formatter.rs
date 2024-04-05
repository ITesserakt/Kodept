use std::io::{Error, Write};
use std::ops::Deref;

use codespan_reporting::diagnostic::Severity;
use derive_more::From;

use kodept_ast::graph::{ChangeSet, GenericASTNode};
use kodept_ast::utils::Execution;
use kodept_ast::utils::Execution::Completed;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_core::Named;

use crate::error::report::ReportMessage;
use crate::Macro;
use crate::traits::Context;

pub struct ASTFormatter<W: Write> {
    writer: W,
    indent: usize,
}

impl<W: Write> Named for ASTFormatter<W> {}

#[derive(From)]
pub struct IOError(Error);

impl<T: Into<IOError>> From<T> for ReportMessage {
    fn from(value: T) -> Self {
        Self::new(Severity::Bug, "IO000", value.into().0.to_string())
    }
}

impl<W: Write> ASTFormatter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer, indent: 0 }
    }
}

impl<W: Write> Macro for ASTFormatter<W> {
    type Error = IOError;
    type Node = GenericASTNode;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        _: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        let node_data = node.deref();
        let f = &mut self.writer;

        match side {
            VisitSide::Entering => {
                writeln!(f, "{}{:?} {{", "  ".repeat(self.indent), node_data).map_err(IOError)?;
                self.indent += 1;
            }
            VisitSide::Leaf => {
                writeln!(f, "{}{:?};", "  ".repeat(self.indent), node_data).map_err(IOError)?
            }
            VisitSide::Exiting => {
                self.indent -= 1;
                writeln!(f, "{}}}", "  ".repeat(self.indent)).map_err(IOError)?;
            }
        }

        Completed(ChangeSet::new())
    }
}
