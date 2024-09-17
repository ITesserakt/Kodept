use crate::hlist::{FromHList, HCons, HList, HNil};
use kodept_ast::graph::{ChangeSet, GenericNodeId, SubEnum};
use kodept_ast::visit_side::VisitSide;
use kodept_macros::context::{Context, FileId};
use kodept_macros::error::report::{IntoSpannedReportMessage, Report};
use kodept_macros::execution::Execution;
use kodept_macros::execution::Execution::{Completed, Failed, Skipped};
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::Macro;

pub mod common;
pub mod pipeline;

struct Pack<'a, C> {
    node_id: GenericNodeId,
    side: VisitSide,
    ctx: &'a mut C,
}

trait RunMacros: HList {
    type Ctx<'a>;

    fn apply(&mut self, pack: Pack<Self::Ctx<'_>>) -> Execution<Vec<Report<FileId>>, ChangeSet>;
}

impl RunMacros for HNil {
    type Ctx<'a> = Context<'a>;

    #[inline]
    fn apply(&mut self, _: Pack<Self::Ctx<'_>>) -> Execution<Vec<Report<FileId>>, ChangeSet> {
        Skipped
    }
}

impl<N, Head, Tail> RunMacros for HCons<Head, Tail>
where
    for<'a> Head: Macro<Node = N, Ctx<'a> = Context<'a>>,
    for<'a> Tail: RunMacros<Ctx<'a> = Context<'a>>,
    Head::Error: IntoSpannedReportMessage,
    N: SubEnum,
{
    type Ctx<'a> = Context<'a>;

    #[inline]
    fn apply(&mut self, pack: Pack<Self::Ctx<'_>>) -> Execution<Vec<Report<FileId>>, ChangeSet> {
        let head = if N::VARIANTS.contains(&pack.ctx.describe(pack.node_id)) {
            let guard = VisitGuard::new(pack.node_id.coerce(), pack.side);
            self.head.apply(guard, pack.ctx)
        } else {
            Skipped
        };
        let file_id = pack.ctx.current_file.id;
        let tail = self.tail.apply(pack);

        match (head, tail) {
            (Failed(e1), Failed(mut e2)) => {
                e2.push(Report::from_message(file_id, e1));
                Failed(e2)
            }
            (_, Failed(e)) => Failed(e),
            (Failed(e), _) => Failed(vec![Report::from_message(file_id, e)]),
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

fn run_macros<M>(ctx: &mut Context, macros: &mut M)
where
    for<'a> M: RunMacros<Ctx<'a> = Context<'a>>
{
    let mut changes = ChangeSet::new();
    let mut iter = ctx.ast.dfs().detach();

    while let Some((node_id, side)) = iter.next(&ctx.ast) {
        match macros.apply(Pack { node_id, side, ctx }) {
            Failed(e) => e.into_iter().for_each(|it| ctx.reports.push_report(it)),
            Completed(next) => changes.extend(next),
            Skipped => continue,
        }
    }
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
    ) -> Option<O>
    where
        for<'a> Self::Inputs: RunMacros<Ctx<'a> = Context<'a>>,
    {
        let mut contents = self.into_contents();
        run_macros(ctx, &mut contents);
        if ctx.reports.has_errors() {
            None
        } else {
            Some(O::from_hlist(contents))
        }
    }
}
