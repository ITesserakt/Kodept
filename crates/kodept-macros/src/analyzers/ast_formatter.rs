use std::cell::{Cell, RefCell};
use std::io::{Error, Write};

use codespan_reporting::diagnostic::Severity;

use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::TraversingResult;
use kodept_ast_graph::generic_node::GenericASTNode;
use kodept_core::Named;

use crate::analyzer::Analyzer;
use crate::error::report::ReportMessage;
use crate::traits::{Context, UnrecoverableError};

pub struct ASTFormatter<W: Write> {
    writer: RefCell<W>,
    indent: Cell<usize>,
}

impl<W: Write> Named for ASTFormatter<W> {}

struct IOError(Error);

impl From<IOError> for ReportMessage {
    fn from(value: IOError) -> Self {
        Self::new(Severity::Bug, "IO000", value.0.to_string())
    }
}

impl<W: Write> ASTFormatter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: RefCell::new(writer),
            indent: Cell::new(0),
        }
    }
}

#[inline]
fn report_io_error<'a, 'c, C: Context<'c>>(
    ctx: &'a C,
) -> impl Fn(Error) -> Result<(), UnrecoverableError> + 'a {
    move |e| ctx.report_and_fail(vec![], IOError(e)).map(|_| ())
}

impl<W: Write> Analyzer for ASTFormatter<W> {
    type Error = UnrecoverableError;
    type Node<'n> = &'n GenericASTNode;

    fn analyze<'n, 'c, C: Context<'c>>(
        &self,
        guard: VisitGuard<Self::Node<'n>>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let (node, side) = guard.allow_all();
        let mut f = self.writer.borrow_mut();

        match side {
            VisitSide::Entering => {
                write!(f, "{}", "  ".repeat(self.indent.get()))
                    .or_else(report_io_error(context))?;
                self.indent.set(self.indent.get() + 1);
            }
            VisitSide::Exiting => {
                self.indent.set(self.indent.get() - 1);
                write!(f, "{}", "  ".repeat(self.indent.get()))
                    .or_else(report_io_error(context))?;
            }
            _ => {
                write!(f, "{}", "  ".repeat(self.indent.get()))
                    .or_else(report_io_error(context))?;
            }
        }

        match side {
            VisitSide::Entering => writeln!(f, "{:?} {{", node),
            VisitSide::Leaf => writeln!(f, "{:?};", node),
            VisitSide::Exiting => writeln!(f, "}}"),
        }
        .or_else(report_io_error(context))?;

        Ok(())
    }
}
