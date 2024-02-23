use std::any::Any;

use kodept_ast::graph::{ChangeSet, GenericASTNode, GhostToken, RefMut, RefNode};
use kodept_ast::visitor::visit_side::{SkipExt, VisitGuard, VisitSide};
use kodept_core::{ConvertibleToRef, Named};

use crate::Macro;
use crate::traits::Context;

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
    ) -> Result<ChangeSet, Self::Error>;
}

pub type BoxedMacro<C, E> = Box<dyn ErasedMacro<C, Error = E>>;

impl<C, T, E> CanErase<C> for T
where
    T: Macro<Error = E> + Named + 'static,
    C: Context,
    GenericASTNode: ConvertibleToRef<T::Node>,
{
    type Error = E;

    fn erase(self) -> BoxedMacro<C, Self::Error> {
        Box::new(self)
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<C, T, E> ErasedMacro<C> for T
where
    C: Context,
    T: Macro<Error = E> + Named + 'static,
    GenericASTNode: ConvertibleToRef<T::Node>,
{
    fn transform(
        &mut self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Result<ChangeSet, Self::Error> {
        let Some(_) = node.ro(token).try_as_ref() else {
            return Ok(ChangeSet::new());
        };
        <Self as Macro>::transform(
            self,
            VisitGuard::new(side, RefMut::new(node), token),
            context,
        )
        .skipped()
    }
}
