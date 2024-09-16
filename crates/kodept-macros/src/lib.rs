#![feature(try_trait_v2)]

use extend::ext;
use tracing::warn;

use crate::context::Context;
use crate::execution::Execution;
use crate::execution::Execution::Skipped;
use crate::visit_guard::VisitGuard;
use kodept_ast::graph::{AnyNode, ChangeSet, NodeId};
use kodept_core::{ConvertibleToMut, ConvertibleToRef};

pub mod context;
pub mod default;
pub mod error;
pub mod execution;
pub mod visit_guard;

pub fn warn_about_broken_rlt<T>() {
    warn!(
        expected = std::any::type_name::<T>(),
        "Skipping some checks because node in RLT either doesn't exist or has different type."
    );
}

pub trait Macro {
    type Error;
    /// Node to transform
    type Node: TryFrom<AnyNode>;
    type Ctx<'a>;

    #[allow(unused_variables)]
    #[inline(always)]
    fn apply<'a>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'a>,
    ) -> Execution<Self::Error, ChangeSet> {
        Skipped
    }
}

#[ext(name = MacroExt)]
pub impl<M> M
where
    for<'a> M: Macro<Ctx<'a> = Context<'a>>,
{
    fn resolve<'a, 'b>(&self, id: NodeId<M::Node>, ctx: &'b M::Ctx<'a>) -> &'b M::Node
    where
        AnyNode: ConvertibleToRef<M::Node>,
    {
        ctx.ast.get(id).unwrap()
    }

    fn resolve_mut<'a, 'b>(&self, id: NodeId<M::Node>, ctx: &'b mut M::Ctx<'a>) -> &'b mut M::Node
    where
        AnyNode: ConvertibleToMut<M::Node>,
    {
        ctx.ast.get_mut(id).unwrap()
    }
}
