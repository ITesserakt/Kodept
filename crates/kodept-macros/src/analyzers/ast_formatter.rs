use std::io::{Error, Write};
use std::ops::Deref;

use codespan_reporting::diagnostic::Severity;
use derive_more::From;

use kodept_ast::graph::GenericASTNode;
use kodept_ast::rlt_accessor::RLTFamily;
use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::TraversingResult;
use kodept_core::structure::Located;
use kodept_core::Named;

use crate::analyzer::Analyzer;
use crate::error::report::{ReportMessage, ResultTEExt};
use crate::traits::{Context, UnrecoverableError};

pub struct ASTFormatter<W: Write> {
    writer: W,
    indent: usize,
}

impl<W: Write> Named for ASTFormatter<W> {}

#[derive(From)]
struct IOError(Error);

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

#[inline]
fn report_io_error<'a>(
    ctx: &'a impl Context,
) -> impl Fn(Error) -> Result<(), UnrecoverableError> + 'a {
    move |e| ctx.report_and_fail(vec![], IOError(e))
}

impl<W: Write> Analyzer for ASTFormatter<W> {
    type Error = UnrecoverableError;
    type Node = GenericASTNode;

    fn analyze(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> TraversingResult<Self::Error> {
        let (node, side) = guard.allow_all();
        let node_data = node.deref();
        let f = &mut self.writer;

        match side {
            VisitSide::Entering => {
                write!(f, "{}{:?} {{", "  ".repeat(self.indent), node_data).report_errors(
                    node_data,
                    context,
                    |it: &RLTFamily| vec![it.location()],
                );
                self.indent += 1;
            }
            VisitSide::Exiting => {
                self.indent -= 1;
                write!(f, "{}{:?};", "  ".repeat(self.indent), node_data)
                    .or_else(report_io_error(context))?;
            }
            _ => {
                write!(f, "{}}}", "  ".repeat(self.indent)).or_else(report_io_error(context))?;
            }
        }

        Ok(())
    }
}
