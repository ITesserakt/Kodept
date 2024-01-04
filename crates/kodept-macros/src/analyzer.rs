use kodept_ast::visitor::visit_side::VisitGuard;
use kodept_ast::visitor::TraversingResult;
use kodept_ast_graph::generic_node::GenericASTNode;
use kodept_core::code_point::CodePoint;

use crate::traits::{Context, UnrecoverableError};

pub trait Analyzer {
    type Error: Into<UnrecoverableError>;
    type Node<'n>: TryFrom<&'n GenericASTNode>;

    fn analyze<'n, 'c, C: Context<'c>>(
        &self,
        guard: VisitGuard<Self::Node<'n>>,
        context: &mut C,
    ) -> TraversingResult<Self::Error>;
}

pub trait AccessExt {
    fn or_unknown(self) -> Vec<CodePoint>;
}

impl AccessExt for Option<CodePoint> {
    fn or_unknown(self) -> Vec<CodePoint> {
        self.map_or(vec![], |it| vec![it])
    }
}

impl AccessExt for Option<Vec<CodePoint>> {
    fn or_unknown(self) -> Vec<CodePoint> {
        self.unwrap_or(vec![])
    }
}
