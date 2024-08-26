use tracing::warn;

use kodept_ast::graph::{AnyNode, ChangeSet};
use kodept_ast::utils::Execution;
use kodept_ast::utils::Execution::Skipped;
use kodept_ast::visit_side::VisitGuard;

use crate::error::report::ReportMessage;
use crate::traits::Context;

pub mod default;
pub mod error;
pub mod traits;

pub fn warn_about_broken_rlt<T>() {
    warn!(
        expected = std::any::type_name::<T>(),
        "Skipping some checks because node in RLT either doesn't exist or has different type."
    );
}

pub trait Macro {
    type Error: Into<ReportMessage>;
    type Node: TryFrom<AnyNode>;

    #[allow(unused_variables)]
    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        Skipped
    }
}

impl<M: Macro> Macro for &mut M {
    type Error = M::Error;
    type Node = M::Node;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        M::transform(self, guard, context)
    }
}
