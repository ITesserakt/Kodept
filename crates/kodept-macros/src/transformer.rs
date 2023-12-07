use crate::traits::{Context, UnrecoverableError};
use kodept_ast::visitor::visit_side::VisitGuard;
use kodept_ast::visitor::TraversingResult;

pub trait Transformer {
    type Error: Into<UnrecoverableError>;
    type Node;

    fn transform<'c, C: Context<'c>>(
        &self,
        guard: VisitGuard<&mut Self::Node>,
        context: &mut C,
    ) -> TraversingResult<Self::Error>;
}
