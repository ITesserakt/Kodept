use kodept_ast::graph::GenericASTNode;
use kodept_ast::visitor::visit_side::{MutVisitGuard, VisitGuard};
use kodept_ast::visitor::TraversingResult;

use crate::traits::{Context, UnrecoverableError};

pub trait Transformer {
    type Error: Into<UnrecoverableError>;
    type Node<'n>: TryFrom<&'n mut GenericASTNode>;

    fn transform<'n, 'c>(
        &self,
        guard: MutVisitGuard<Self::Node<'n>>,
        context: &mut impl Context<'c>,
    ) -> TraversingResult<Self::Error>;
}
