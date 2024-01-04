use kodept_ast::visitor::visit_side::VisitGuard;
use kodept_ast::visitor::TraversingResult;
use kodept_ast_graph::generic_node::GenericASTNode;

use crate::traits::{Context, UnrecoverableError};

pub trait Transformer {
    type Error: Into<UnrecoverableError>;
    type Node<'n>: TryFrom<&'n mut GenericASTNode>;

    fn transform<'n, 'c>(
        &self,
        guard: VisitGuard<Self::Node<'n>>,
        context: &mut impl Context<'c>,
    ) -> TraversingResult<Self::Error>;
}
