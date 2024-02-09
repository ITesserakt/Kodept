use std::fmt::{Debug, Formatter};

use derive_more::From;

use kodept_ast::graph::{ChangeSet, GenericASTNode, GhostToken, RefMut, RefNode};
use kodept_ast::visitor::visit_side::{SkipExt, VisitGuard, VisitSide};
use kodept_core::{ConvertibleToRef, Named};

use crate::analyzer::Analyzer;
use crate::traits::{Context, UnrecoverableError};
use crate::transformer::Transformer;

pub trait ErasedAnalyzer<C: Context>: Named {
    type Error;

    fn analyze(
        &mut self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Result<(), Self::Error>;

    fn erase(self) -> Erased<C, Self::Error>
    where
        Self: Sized + 'static,
    {
        Erased::Analyzer(Box::new(self))
    }
}

pub trait ErasedTransformer<C: Context>: Named {
    type Error;

    fn transform(
        &self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Result<ChangeSet, Self::Error>;

    fn erase(self) -> Erased<C, Self::Error>
    where
        Self: Sized + 'static,
    {
        Erased::Transformer(Box::new(self))
    }
}

type BoxedTransformer<C, E> = Box<dyn ErasedTransformer<C, Error = E>>;
type BoxedAnalyzer<C, E> = Box<dyn ErasedAnalyzer<C, Error = E>>;

#[derive(From)]
pub enum Erased<C: Context, E> {
    Transformer(BoxedTransformer<C, E>),
    Analyzer(BoxedAnalyzer<C, E>),
}

impl<C: Context, E> Debug for Erased<C, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<C: Context, E> Named for Erased<C, E> {
    fn name(&self) -> &'static str {
        match self {
            Erased::Transformer(x) => x.name(),
            Erased::Analyzer(x) => x.name(),
        }
    }
}

impl<C, T: Named> ErasedTransformer<C> for T
where
    C: Context,
    T: Transformer + 'static,
    GenericASTNode: ConvertibleToRef<T::Node>,
{
    type Error = UnrecoverableError;

    fn transform(
        &self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Result<ChangeSet, Self::Error> {
        let Some(_) = node.ro(token).try_as_ref() else {
            return Ok(ChangeSet::new());
        };
        <Self as Transformer>::transform(
            self,
            VisitGuard::new(side, RefMut::new(node), token),
            context,
        )
        .skipped()
        .map_err(|e| e.into())
    }

    fn erase(self) -> Erased<C, Self::Error> {
        Erased::Transformer(Box::new(self))
    }
}

impl<'c, C, A: Named> ErasedAnalyzer<C> for A
where
    C: Context,
    A: Analyzer + 'static,
    GenericASTNode: ConvertibleToRef<A::Node>,
{
    type Error = UnrecoverableError;

    fn analyze(
        &mut self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Result<(), Self::Error> {
        let Some(_) = node.ro(token).try_as_ref() else {
            return Ok(());
        };
        <Self as Analyzer>::analyze(
            self,
            VisitGuard::new(side, RefMut::new(node), token),
            context,
        )
        .skipped()
        .map_err(|e| e.into())
    }

    fn erase(self) -> Erased<C, Self::Error> {
        Erased::Analyzer(Box::new(self))
    }
}
