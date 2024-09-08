use crate::hlist::{FromHList, HCons, HList, HNil};
use crate::macro_context::{MacroContext, TreeTraversal};
use kodept_ast::graph::{ChangeSet, GenericNodeId, SubEnum};
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::VisitSide;
use kodept_macros::context::{Context, Reporter, SyntaxProvider};
use kodept_macros::error::report::ReportMessage;
use kodept_macros::unrecoverable_error::UnrecoverableError;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::Macro;
use std::convert::Infallible;
use Execution::{Completed, Failed, Skipped};

pub mod capabilities;
pub mod common;
pub mod pipeline;

struct Pack<'a, C> {
    node_id: GenericNodeId,
    side: VisitSide,
    ctx: &'a mut C,
}

trait RunMacros<Capability>: HList {
    type Error: Into<ReportMessage>;

    fn apply<C: Context<Capability>>(&mut self, pack: Pack<C>)
        -> Execution<Self::Error, ChangeSet>;
}

impl<Capability> RunMacros<Capability> for HNil {
    type Error = Infallible;

    #[inline]
    fn apply<C: Context<Capability>>(&mut self, _: Pack<C>) -> Execution<Self::Error, ChangeSet> {
        Skipped
    }
}

impl<N, Head, Tail, Capability> RunMacros<Capability> for HCons<Head, Tail>
where
    Head: Macro<Capability, Node = N, Error: Into<ReportMessage>>,
    Tail: RunMacros<Capability>,
    N: SubEnum,
    Capability: SyntaxProvider
{
    type Error = ReportMessage;

    #[inline]
    fn apply<C: Context<Capability>>(&mut self, pack: Pack<C>) -> Execution<Self::Error, ChangeSet> {
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

fn run_macros<C>(
    ctx: &mut MacroContext<C>,
    macros: &mut impl RunMacros<C>,
) -> Result<(), UnrecoverableError>
where
    C: TreeTraversal + Reporter
{
    let mut changes = ChangeSet::new();
    let mut iter = ctx.detached_iter();

    while let Some((node_id, side)) = iter.next(ctx.get_tree()) {
        match macros.apply(Pack {
            node_id,
            side,
            ctx,
        }) {
            Failed(e) => {
                let location = vec![];
                ctx.report_and_fail(location, e)?
            }
            Completed(next) => changes.extend(next),
            Skipped => continue,
        }
    }

    // FIXME
    Ok(())
}

pub trait Step<Capability>
where
    Self: Sized,
{
    #[allow(private_bounds)]
    type Inputs: RunMacros<Capability>;

    fn into_contents(self) -> Self::Inputs;

    fn apply_with_context<O: FromHList<Self::Inputs>>(
        self,
        ctx: &mut MacroContext<Capability>,
    ) -> Result<O, UnrecoverableError>
    where
        Capability: TreeTraversal + Reporter,
    {
        let mut contents = self.into_contents();
        run_macros(ctx, &mut contents)?;
        Ok(O::from_hlist(contents))
    }
}
