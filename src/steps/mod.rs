use crate::hlist::{FromHList, HCons, HList, HNil};
use kodept_ast::graph::{ChangeSet, GenericNodeId, SubEnum};
use kodept_ast::visit_side::VisitSide;
use kodept_macros::context::{Context, FileId};
use kodept_macros::error::report::{IntoSpannedReportMessage, Report, ReportMessage};
use kodept_macros::execution::Execution;
use kodept_macros::execution::Execution::{Completed, Failed, Skipped};
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::Macro;
use std::convert::Infallible;

pub mod common;
pub mod pipeline;

struct Pack<'a, C> {
    node_id: GenericNodeId,
    side: VisitSide,
    ctx: &'a mut C,
}

trait RunMacros: HList {
    type Error: Into<ReportMessage>;
    type Ctx<'a>;

    fn apply(&mut self, pack: Pack<Self::Ctx<'_>>) -> Execution<Self::Error, ChangeSet>;
}

impl RunMacros for HNil {
    type Error = Infallible;
    type Ctx<'a> = Context<'a>;

    #[inline]
    fn apply(&mut self, _: Pack<Self::Ctx<'_>>) -> Execution<Self::Error, ChangeSet> {
        Skipped
    }
}

impl<N, Head, Tail> RunMacros for HCons<Head, Tail>
where
    for<'a> Head: Macro<Node = N, Error: Into<ReportMessage>, Ctx<'a> = Context<'a>>,
    for<'a> Tail: RunMacros<Ctx<'a> = Context<'a>>,
    N: SubEnum,
{
    type Error = ReportMessage;
    type Ctx<'a> = Context<'a>;

    #[inline]
    fn apply(&mut self, pack: Pack<Self::Ctx<'_>>) -> Execution<Self::Error, ChangeSet> {
        let head = if N::VARIANTS.contains(&pack.ctx.describe(pack.node_id)) {
            let guard = VisitGuard::new(pack.node_id.coerce(), pack.side);
            self.head.apply(guard, pack.ctx)
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

fn run_macros<M>(ctx: &mut Context, macros: &mut M) -> Result<(), Report<FileId>>
where
    for<'a> M: RunMacros<Ctx<'a> = Context<'a>>,
    M::Error: IntoSpannedReportMessage,
{
    let mut changes = ChangeSet::new();
    let mut iter = ctx.ast.dfs().detach();

    while let Some((node_id, side)) = iter.next(&ctx.ast) {
        match macros.apply(Pack { node_id, side, ctx }) {
            Failed(e) => ctx.report_and_fail(e)?,
            Completed(next) => changes.extend(next),
            Skipped => continue,
        }
    }

    Ok(())
}

pub trait Step
where
    Self: Sized,
{
    type Inputs;

    fn into_contents(self) -> Self::Inputs;

    #[allow(private_bounds)]
    fn apply_with_context<O: FromHList<Self::Inputs>>(
        self,
        ctx: &mut Context,
    ) -> Result<O, Report<FileId>>
    where 
        for<'a> Self::Inputs: RunMacros<Ctx<'a> = Context<'a>>
    {
        let mut contents = self.into_contents();
        run_macros(ctx, &mut contents)?;
        Ok(O::from_hlist(contents))
    }
}
