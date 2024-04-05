use std::any::Any;

use kodept_ast::graph::{ChangeSet, GenericASTNode, GhostToken, RefMut, RefNode};
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_core::ConvertibleToRef;
use kodept_core::structure::Located;

use crate::error::report::ReportMessage;
use crate::Macro;
use crate::traits::{Context, UnrecoverableError};

pub trait CanErase<C: Context> {
    type Error;

    fn erase(self) -> BoxedMacro<C, Self::Error>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait ErasedMacro<C: Context>: CanErase<C> {
    fn transform(
        &mut self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Execution<Self::Error, ChangeSet>;
}

pub type BoxedMacro<C, E> = Box<dyn ErasedMacro<C, Error = E>>;

impl<C, T, E: Into<ReportMessage>> CanErase<C> for T
where
    T: Macro<Error = E> + 'static,
    C: Context,
    GenericASTNode: ConvertibleToRef<T::Node>,
{
    type Error = UnrecoverableError;

    fn erase(self) -> BoxedMacro<C, Self::Error> {
        Box::new(self)
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<C, T, E: Into<ReportMessage>> ErasedMacro<C> for T
where
    C: Context,
    T: Macro<Error = E> + 'static,
    GenericASTNode: ConvertibleToRef<T::Node>,
{
    fn transform(
        &mut self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Execution<Self::Error, ChangeSet> {
        let Some(_) = node.ro(token).try_as_ref() else {
            return Execution::Skipped;
        };
        let exec = <Self as Macro>::transform(
            self,
            VisitGuard::new(side, RefMut::new(node), token),
            context,
        );

        match exec {
            Execution::Failed(e) => {
                let location = context
                    .access_unknown(node.ro(token))
                    .map_or(vec![], |it| vec![it.location()]);
                match context.report_and_fail::<E, ()>(location, e) {
                    Ok(_) => unreachable!(),
                    Err(report) => Execution::Failed(report),
                }
            }
            Execution::Completed(x) => Execution::Completed(x),
            Execution::Skipped => Execution::Skipped,
        }
    }
}
