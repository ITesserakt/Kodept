use crate::traversing::OptionalContext::Defined;
use crate::utils::graph::topological_layers;
use itertools::{Either, Itertools};
use kodept_macros::erased::{Erased, ErasedAnalyzer};
use kodept_macros::traits::Context;
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::{DiGraph, NodeIndex};
use std::ops::{Deref, DerefMut};

type DefaultErased<'c, C, E> = Erased<'c, C, E>;

pub struct TraverseSet<'c, C, E>
where
    C: Context<'c>,
{
    inner: DiGraph<DefaultErased<'c, C, E>, ()>,
}

impl<'c, C: Context<'c>, E> Default for TraverseSet<'c, C, E> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

pub trait Traversable<'c, C: Context<'c>, E> {
    fn traverse(&self, context: C) -> Result<C, (Vec<E>, C)>;
}

impl<'c, C, E> TraverseSet<'c, C, E>
where
    C: Context<'c>,
{
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn add_independent(&mut self, item: Erased<'c, C, E>) -> NodeIndex {
        self.inner.add_node(item)
    }

    pub fn add_link(&mut self, from: NodeIndex, to: NodeIndex) {
        if from == to {
            panic!("Cannot add link to itself");
        }
        self.inner.add_edge(from, to, ());
        if is_cyclic_directed(&self.inner) {
            panic!("Cannot add link that produces a cycle");
        }
    }

    pub fn add_dependent(&mut self, from: &[NodeIndex], item: Erased<'c, C, E>) -> NodeIndex {
        let id = self.inner.add_node(item);
        from.iter().for_each(|&it| {
            self.inner.add_edge(it, id, ());
        });
        id
    }

    #[allow(clippy::manual_try_fold)]
    fn run_analyzers(
        context: &mut C,
        analyzers: &[&dyn ErasedAnalyzer<'c, C, Error = E>],
    ) -> Result<(), Vec<E>> {
        context
            .tree()
            .iter()
            .map(|(node, side)| {
                analyzers
                    .iter()
                    .try_for_each(|a| a.analyze(node, side, context))
            })
            .fold(Ok(()), |acc, next| match (acc, next) {
                (Ok(_), Ok(_)) => Ok(()),
                (Ok(_), Err(e)) => Err(vec![e]),
                (Err(e), Ok(_)) => Err(e),
                (Err(mut e1), Err(e2)) => {
                    e1.push(e2);
                    Err(e1)
                }
            })
    }
}

impl<'c, C: Context<'c>, E> Traversable<'c, C, E> for TraverseSet<'c, C, E> {
    fn traverse(&self, mut context: C) -> Result<C, (Vec<E>, C)> {
        let sorted = topological_layers(&self.inner);
        for layer in sorted {
            let (transformers, analyzers): (Vec<_>, Vec<_>) = layer
                .into_iter()
                .map(|x| &self.inner[x])
                .partition_map(|e| match e {
                    Erased::Transformer(x) => Either::Left(x),
                    Erased::Analyzer(x) => Either::Right(x.as_ref()),
                });

            let analyzer_result = TraverseSet::run_analyzers(&mut context, &analyzers);
            match analyzer_result {
                Ok(_) => {}
                Err(e) => return Err((e, context)),
            };
        }
        Ok(context)
    }
}

#[derive(Default)]
enum OptionalContext<C> {
    Defined(C),
    #[default]
    None,
}

impl<C> OptionalContext<C> {
    pub const TAKEN_MSG: &'static str = "Cannot get context that was taken";

    pub fn into_inner(self) -> C {
        match self {
            Defined(c) => c,
            OptionalContext::None => unreachable!("{}", Self::TAKEN_MSG),
        }
    }
}

impl<C> Deref for OptionalContext<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        match self {
            Defined(c) => c,
            OptionalContext::None => unreachable!("{}", OptionalContext::<C>::TAKEN_MSG),
        }
    }
}

impl<C> DerefMut for OptionalContext<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Defined(c) => c,
            OptionalContext::None => unreachable!("{}", OptionalContext::<C>::TAKEN_MSG),
        }
    }
}
