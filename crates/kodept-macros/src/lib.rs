use tracing::warn;

use crate::context::{Context, SyntaxProvider};
use crate::visit_guard::VisitGuard;
use kodept_ast::graph::{AnyNode, ChangeSet, NodeId};
use kodept_ast::utils::Execution;
use kodept_ast::utils::Execution::Skipped;
use kodept_core::{ConvertibleToMut, ConvertibleToRef};

pub mod context;
pub mod default;
pub mod error;
pub mod unrecoverable_error;
pub mod visit_guard;

pub fn warn_about_broken_rlt<T>() {
    warn!(
        expected = std::any::type_name::<T>(),
        "Skipping some checks because node in RLT either doesn't exist or has different type."
    );
}

pub trait Macro<Capability> {
    type Error;
    /// Node to transform
    type Node: TryFrom<AnyNode>;

    #[allow(unused_variables)]
    #[inline(always)]
    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut impl Context<Capability>,
    ) -> Execution<Self::Error, ChangeSet> {
        Skipped
    }

    fn resolve<'a>(
        &self,
        id: NodeId<Self::Node>,
        ctx: &'a impl Context<Capability>,
    ) -> &'a Self::Node
    where
        Capability: SyntaxProvider + 'a,
        AnyNode: ConvertibleToRef<Self::Node>,
    {
        ctx.get(id).unwrap()
    }

    fn resolve_mut<'a>(
        &self,
        id: NodeId<Self::Node>,
        ctx: &'a mut impl Context<Capability>,
    ) -> &'a mut Self::Node
    where
        Capability: SyntaxProvider + 'a,
        AnyNode: ConvertibleToMut<Self::Node>,
    {
        ctx.get_mut(id).unwrap()
    }
}
