use std::fmt::{Debug, Formatter};

use derive_more::From;

use kodept_ast::graph::{ChangeSet, GenericASTNode, GhostToken, RefMut, RefNode};
use kodept_ast::visitor::visit_side::{SkipExt, VisitGuard, VisitSide};
use kodept_core::{ConvertibleToRef, Named};

use crate::analyzer::Analyzer;
use crate::traits::{Context, UnrecoverableError};
use crate::transformer::Transformer;

pub trait ErasedAnalyzer<'c, C>: Named
where
    C: Context<'c>,
{
    type Error;

    fn analyze(
        &mut self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Result<(), Self::Error>;

    fn erase(self) -> Erased<'c, C, Self::Error>
    where
        Self: Sized + 'static,
    {
        Erased::Analyzer(Box::new(self))
    }
}

pub trait ErasedTransformer<'c, C>: Named
where
    C: Context<'c>,
{
    type Error;

    fn transform(
        &self,
        node: RefNode,
        side: VisitSide,
        token: &mut GhostToken,
        context: &mut C,
    ) -> Result<ChangeSet, Self::Error>;

    fn erase(self) -> Erased<'c, C, Self::Error>
    where
        Self: Sized + 'static,
    {
        Erased::Transformer(Box::new(self))
    }
}

type BoxedTransformer<'c, C, E> = Box<dyn ErasedTransformer<'c, C, Error = E>>;
type BoxedAnalyzer<'c, C, E> = Box<dyn ErasedAnalyzer<'c, C, Error = E>>;

#[derive(From)]
pub enum Erased<'c, C, E>
where
    C: Context<'c>,
{
    Transformer(BoxedTransformer<'c, C, E>),
    Analyzer(BoxedAnalyzer<'c, C, E>),
}

impl<'c, C, E> Debug for Erased<'c, C, E>
where
    C: Context<'c>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'c, C: Context<'c>, E> Named for Erased<'c, C, E> {
    fn name(&self) -> &'static str {
        match self {
            Erased::Transformer(x) => x.name(),
            Erased::Analyzer(x) => x.name(),
        }
    }
}

impl<'c, C, T: Named> ErasedTransformer<'c, C> for T
where
    C: Context<'c>,
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

    fn erase(self) -> Erased<'c, C, Self::Error> {
        Erased::Transformer(Box::new(self))
    }
}

impl<'c, C, A: Named> ErasedAnalyzer<'c, C> for A
where
    C: Context<'c>,
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

    fn erase(self) -> Erased<'c, C, Self::Error> {
        Erased::Analyzer(Box::new(self))
    }
}
