use std::io::{Error, Write};
use thiserror::Error;
use kodept_ast::graph::{AnyNode};
use kodept_ast::visit_side::VisitSide;

use crate::context::Context;
use crate::execution::Execution;
use crate::execution::Execution::Completed;
use crate::visit_guard::VisitGuard;
use crate::{Macro, MacroExt};

pub struct ASTFormatter<W: Write> {
    writer: W,
    indent: usize,
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct IOError(Error);

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

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Execution<Self::Error> {
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

        Completed(())
    }
}
