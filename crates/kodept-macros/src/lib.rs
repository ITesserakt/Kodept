use tracing::warn;

use kodept_ast::graph::{ChangeSet, GenericASTNode};
use kodept_ast::visitor::visit_side::{Skip, VisitGuard};

use crate::traits::{Context, UnrecoverableError};

pub mod default;
pub mod erased;
pub mod error;
pub mod traits;
pub mod transformer;

pub fn warn_about_broken_rlt<T>() {
    warn!(
        expected = std::any::type_name::<T>(),
        "Skipping some checks because node in RLT either doesn't exist or has different type."
    );
}

pub trait Macro {
    type Error: Into<UnrecoverableError>;
    type Node: TryFrom<GenericASTNode>;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Result<ChangeSet, Skip<Self::Error>>;
}
