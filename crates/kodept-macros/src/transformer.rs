use kodept_ast::graph::{ChangeSet, GenericASTNode};
use kodept_ast::visitor::visit_side::{Skip, VisitGuard};

use crate::traits::{Context, UnrecoverableError};

pub trait Transformer {
    type Error: Into<UnrecoverableError>;
    type Node: TryFrom<GenericASTNode>;

    fn transform(
        &self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Result<ChangeSet, Skip<Self::Error>>;
}
