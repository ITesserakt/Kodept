use std::convert::Infallible;

use kodept_ast::graph::{AnyNode, ChangeSet, PermTkn, RefNode, TypedNodeCell};
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_core::structure::Located;
use kodept_core::ConvertibleToRef;
use kodept_macros::error::report::ReportMessage;
use kodept_macros::traits::{Context, MutableContext, UnrecoverableError};
use kodept_macros::Macro;
use Execution::{Completed, Failed, Skipped};

use crate::steps::hlist::{HCons, HList, HNil};

pub mod common;
pub mod hlist;
pub mod pipeline;

struct Pack<'arena, 'token, C: Context> {
    node: RefNode<'arena>,
    token: &'token mut PermTkn,
    side: VisitSide,
    ctx: &'arena mut C,
}

trait RunMacros: HList {
    type Error: Into<ReportMessage>;

    fn apply<C: Context>(&mut self, pack: Pack<C>) -> Execution<Self::Error, ChangeSet>;
}

impl RunMacros for HNil {
    type Error = Infallible;

    #[inline]
    fn apply<C: Context>(&mut self, _: Pack<C>) -> Execution<Self::Error, ChangeSet> {
        Skipped
    }
}

impl<N, Head, Tail> RunMacros for HCons<Head, Tail>
where
    Head: Macro<Node = N>,
    Tail: RunMacros,
    AnyNode: ConvertibleToRef<N>,
{
    type Error = ReportMessage;

    #[inline]
    fn apply<C: Context>(&mut self, pack: Pack<C>) -> Execution<Self::Error, ChangeSet> {
        let head = if pack.node.ro(pack.token).try_as_ref().is_some() {
            let guard = VisitGuard::new(pack.side, TypedNodeCell::new(pack.node), pack.token);
            self.head.transform(guard, pack.ctx)
        } else {
            Skipped
        };
        let tail = self.tail.apply(pack);
        
        match (head, tail) {
            (Failed(e), _) => Failed(e.into()),
            (_, Failed(e)) => Failed(e.into()),
            (Skipped, Skipped) => Skipped,
            (Completed(full), Skipped) => Completed(full),
            (Skipped, Completed(full)) => Completed(full),
            (Completed(mut part1), Completed(part2)) => {
                part1.extend(part2);
                Completed(part1)
            }
        }
    }
}

fn run_macros(
    context: &mut impl MutableContext,
    macros: &mut impl RunMacros,
) -> Result<(), UnrecoverableError> {
    let mut token = PermTkn::new();
    let mut changes = ChangeSet::new();

    for (node, side) in context.tree().upgrade().unwrap().dfs() {
        match macros.apply(Pack {
            node,
            token: &mut token,
            side,
            ctx: context,
        }) {
            Failed(e) => {
                // FIXME
                let location = vec![];
                context.report_and_fail(location, e)?;
            }
            Completed(next) => changes.extend(next),
            Skipped => continue,
        }
    }

    // FIXME
    Ok(())
}

pub trait Step
where
    Self: Sized,
{
    #[allow(private_bounds)]
    type Inputs: RunMacros;

    fn into_contents(self) -> Self::Inputs;

    fn apply_with_context<C: MutableContext>(
        self,
        ctx: &mut C,
    ) -> Result<Self::Inputs, UnrecoverableError> {
        let mut contents = self.into_contents();
        run_macros(ctx, &mut contents)?;
        Ok(contents)
    }
}
