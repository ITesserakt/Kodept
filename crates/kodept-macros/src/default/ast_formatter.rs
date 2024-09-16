use std::io::{Error, Write};

use kodept_ast::graph::{AnyNode, ChangeSet};
use kodept_ast::visit_side::VisitSide;

use crate::context::Context;
use crate::error::report::{ReportMessage, Severity};
use crate::execution::Execution;
use crate::execution::Execution::Completed;
use crate::visit_guard::VisitGuard;
use crate::{Macro, MacroExt};

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

impl<W> Macro for ASTFormatter<W>
where
    W: Write
{
    type Error = IOError;
    type Node = AnyNode;
    type Ctx<'a> = Context<'a>;

    fn apply<'a>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'a>,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        let node = self.resolve(node, ctx);
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
