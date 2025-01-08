use self::ApplicationResult::*;
use crate::hlist::{FromHList, HCons, HList, HNil};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_macros::context::{Context, FileId};
use kodept_macros::error::report::{
    IntoSpannedReportMessage, MessageBehaviour, Report, SpannedReportMessage,
};
use std::borrow::{BorrowMut, Cow};
use std::collections::VecDeque;
use tracing::{debug, warn};

pub mod common;
pub mod pipeline;

/// Represents a possible outcome of macro application
enum ApplicationResult {
    /// Macro produced some erroneous output, but we can continue
    SoftFailure(VecDeque<Report<FileId>>),
    /// Macro produced a *fail-fast* error
    HardFailure {
        collected_reports: VecDeque<Report<FileId>>,
        failed_macro_type: &'static str,
        reason: Cow<'static, str>,
    },
    /// Macro did not execute (wrong side, etc)
    Skipped,
    /// Macro was successfully complete
    Completed,
}

struct Pack<'a, C> {
    node_id: AnyNodeId,
    side: VisitSide,
    ctx: &'a mut C,
}

trait RunMacros: HList {
    type Ctx<'a>;

    fn apply(&mut self, pack: Pack<Self::Ctx<'_>>) -> ApplicationResult;
}

impl Pack<'_, Context<'_>> {
    fn current_node_location(&self) -> Option<CodePoint> {
        Some(self.ctx.rlt.get_unknown(self.node_id)?.location())
    }
}

impl RunMacros for HNil {
    type Ctx<'a> = Context<'a>;

    #[inline]
    fn apply(&mut self, _: Pack<Self::Ctx<'_>>) -> ApplicationResult {
        Skipped
    }
}

impl<N, Head, Tail> RunMacros for HCons<Head, Tail>
where
    for<'a> Head: Macro<Node = N, Ctx<'a> = Context<'a>>,
    for<'a> Tail: RunMacros<Ctx<'a> = Context<'a>>,
    N: Node,
{
    type Ctx<'a> = Context<'a>;

    #[inline]
    fn apply(&mut self, pack: Pack<Self::Ctx<'_>>) -> ApplicationResult {
        let head = if N::VARIANTS.contains(&pack.ctx.describe(pack.node_id)) {
            let guard = VisitGuard::new(pack.node_id.coerce(), pack.side);
            self.head.apply(guard, pack.ctx)
        } else {
            Err(Skip::Skipped)
        };
        let location = pack.current_node_location();
        let file_id = pack.ctx.current_file.id;
        let tail = move || self.tail.apply(pack);
        let into_report = |e: Head::Error| match location {
            None => Report::from_message(file_id, e),
            Some(p) => Report::from_message(file_id, e.into_message().with_node_location(p)),
        };

        match head {
            Ok(()) => match tail() {
                Skipped => Completed,
                x => x,
            },
            Err(Skip::Skipped) => tail(),
            Err(Skip::Failed(error)) => match error.behaviour() {
                MessageBehaviour::FailFast { reason } => HardFailure {
                    collected_reports: VecDeque::from([into_report(error)]),
                    failed_macro_type: std::any::type_name::<Head>(),
                    reason,
                },
                MessageBehaviour::Suppress => match tail() {
                    SoftFailure(mut other) => SoftFailure({
                        other.push_front(into_report(error));
                        other
                    }),
                    HardFailure {
                        mut collected_reports,
                        failed_macro_type,
                        reason,
                    } => HardFailure {
                        collected_reports: {
                            collected_reports.push_front(into_report(error));
                            collected_reports
                        },
                        failed_macro_type,
                        reason,
                    },
                    Skipped | Completed => SoftFailure(VecDeque::from([into_report(error)])),
                },
            },
        }
    }
}

fn run_macros<'a, M, C>(ctx: &mut C, macros: &mut M) -> bool
where
    M: RunMacros<Ctx<'a> = Context<'a>>,
    C: BorrowMut<Context<'a>>,
{
    #[derive(Default, Debug)]
    struct Metrics {
        total: usize,
        skipped: usize,
        completed: usize,
        failed: usize,
    }

    let ctx = ctx.borrow_mut();
    let mut iter = ctx.ast.dfs().detach();
    let mut metrics = Metrics::default();

    while let Some((node_id, side)) = iter.next(&ctx.ast) {
        match macros.apply(Pack { node_id, side, ctx }) {
            SoftFailure(reports) => {
                reports
                    .into_iter()
                    .for_each(|it| ctx.collector.push_report(it));
                metrics.failed += 1;
            }
            HardFailure {
                failed_macro_type,
                reason,
                collected_reports,
            } => {
                warn!(failed_macro_type, "Cannot continue: {reason}");
                collected_reports
                    .into_iter()
                    .for_each(|it| ctx.collector.push_report(it));
                debug!(?metrics);
                return false;
            }
            Skipped => metrics.skipped += 1,
            Completed => metrics.completed += 1,
        };
        metrics.total += 1;
    }

    debug!(?metrics);
    metrics.failed == 0
}

pub trait Step
where
    Self: Sized,
{
    type Inputs;

    fn into_contents(self) -> Self::Inputs;

    #[allow(private_bounds)]
    fn apply_with_context<'a, O: FromHList<Self::Inputs>, C>(self, ctx: &mut C) -> Option<O>
    where
        Self::Inputs: RunMacros<Ctx<'a> = Context<'a>>,
        C: BorrowMut<Context<'a>>,
    {
        let mut contents = self.into_contents();
        match run_macros(ctx, &mut contents) {
            true => Some(O::from_hlist(contents)),
            false => None,
        }
    }

    #[allow(private_bounds)]
    fn run_with_context(self, ctx: &mut Context) -> Option<()>
    where
        for<'a> Self::Inputs: RunMacros<Ctx<'a> = Context<'a>>,
    {
        let mut contents = self.into_contents();
        match run_macros(ctx, &mut contents) {
            true => Some(()),
            false => None,
        }
    }
}
