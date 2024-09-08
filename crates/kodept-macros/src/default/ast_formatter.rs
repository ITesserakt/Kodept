use std::io::{Error, Write};

use codespan_reporting::diagnostic::Severity;

use kodept_ast::graph::{AnyNode, ChangeSet};
use kodept_ast::utils::Execution;
use kodept_ast::utils::Execution::Completed;
use kodept_ast::visit_side::VisitSide;

use crate::context::SyntaxProvider;
use crate::error::report::ReportMessage;
use crate::{context, visit_guard, Macro};

pub struct ASTFormatter<W: Write> {
    writer: W,
    indent: usize,
}

pub struct IOError(Error);

impl From<IOError> for ReportMessage {
    fn from(value: IOError) -> Self {
        Self::new(Severity::Bug, "IO000", value.0.to_string())
    }
}

impl<W: Write> ASTFormatter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer, indent: 0 }
    }
}

impl<W, C> Macro<C> for ASTFormatter<W>
where
    W: Write,
    C: SyntaxProvider
{
    type Error = IOError;
    type Node = AnyNode;

    fn apply(
        &mut self,
        node: visit_guard::VisitGuard<AnyNode>,
        ctx: &mut impl context::Context<C>,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = node.allow_all();
        let node = ctx.get(node);
        let writer = &mut self.writer;

        match side {
            VisitSide::Entering => {
                writeln!(writer, "{}{:?} {{", "  ".repeat(self.indent), node).map_err(IOError)?;
                self.indent += 1;
            }
            VisitSide::Leaf => {
                writeln!(writer, "{}{:?};", "  ".repeat(self.indent), node).map_err(IOError)?
            }
            VisitSide::Exiting => {
                self.indent -= 1;
                writeln!(writer, "{}}}", "  ".repeat(self.indent)).map_err(IOError)?;
            }
        }

        Completed(ChangeSet::new())
    }
}
