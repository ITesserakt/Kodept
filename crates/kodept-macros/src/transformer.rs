use crate::traits::{Context, UnrecoverableError};
use kodept_ast::graph::generic_node::GenericASTNode;
use kodept_ast::visitor::visit_side::VisitGuard;
use kodept_ast::visitor::TraversingResult;

pub trait Transformer {
    type Error: Into<UnrecoverableError>;
    type Node<'n>: TryFrom<&'n mut GenericASTNode>;

    fn transform<'n, 'c, C: Context<'c>>(
        &self,
        guard: VisitGuard<Self::Node<'n>>,
        context: &mut C,
    ) -> TraversingResult<Self::Error>;
}
