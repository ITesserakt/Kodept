use kodept_ast::graph::node_props::Node;
use kodept_ast::graph::{AnyNode, NodeId};
use kodept_ast::utils::Skip;
use kodept_report::prelude::IntoSpannedReportMessage;

mod context;
mod visit_guard;

pub use context::Context;
pub use visit_guard::VisitGuard;

pub trait Macro {
    type Error: IntoSpannedReportMessage;
    type Node: Into<AnyNode>;
    type Ctx<'a>;

    fn apply(
        &mut self,
        node: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Result<(), Skip<Self::Error>>;
}

pub trait MacroExt<N> {
    fn resolve<'a>(id: NodeId<N>, ctx: &'a Context<'_>) -> &'a N;
    fn resolve_mut<'a>(id: NodeId<N>, ctx: &'a mut Context<'_>) -> &'a mut N;
}

impl<'c, N, M> MacroExt<N> for M
where
    M: Macro<Node = N, Ctx<'c> = Context<'c>>,
    N: Node,
{
    fn resolve<'a>(id: NodeId<N>, ctx: &'a Context<'_>) -> &'a N {
        ctx.ast.get(id).unwrap()
    }

    fn resolve_mut<'a>(id: NodeId<N>, ctx: &'a mut Context<'_>) -> &'a mut N {
        ctx.ast.get_mut(id).unwrap()
    }
}
