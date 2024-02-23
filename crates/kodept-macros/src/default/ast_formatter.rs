use std::io::{Error, Write};
use std::ops::Deref;

use codespan_reporting::diagnostic::Severity;
use derive_more::From;

use kodept_ast::graph::{ChangeSet, GenericASTNode};
use kodept_ast::rlt_accessor::RLTFamily;
use kodept_ast::visitor::visit_side::{Skip, VisitGuard, VisitSide};
use kodept_core::Named;
use kodept_core::structure::Located;

use crate::error::report::{ReportMessage, ResultTEExt};
use crate::Macro;
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

impl<W: Write> Macro for ASTFormatter<W> {
    type Error = UnrecoverableError;
    type Node = GenericASTNode;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Result<ChangeSet, Skip<Self::Error>> {
        let (node, side) = guard.allow_all();
        let node_data = node.deref();
        let f = &mut self.writer;

        match side {
            VisitSide::Entering => {
                writeln!(f, "{}{:?} {{", "  ".repeat(self.indent), node_data).report_errors(
                    node_data,
                    context,
                    |it: &RLTFamily| vec![it.location()],
                );
                self.indent += 1;
            }
            VisitSide::Leaf => writeln!(f, "{}{:?};", "  ".repeat(self.indent), node_data)
                .report_errors(node_data, context, |it: &RLTFamily| vec![it.location()]),
            VisitSide::Exiting => {
                self.indent -= 1;
                writeln!(f, "{}}}", "  ".repeat(self.indent)).report_errors(
                    node_data,
                    context,
                    |it: &RLTFamily| vec![it.location()],
                );
            }
        }

        Ok(ChangeSet::empty())
    }
}
