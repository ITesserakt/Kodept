use itertools::{Either, Itertools};
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::{DiGraph, NodeIndex};

use kodept_ast::graph::GhostToken;
use kodept_macros::erased::{Erased, ErasedAnalyzer};
use kodept_macros::traits::Context;

use crate::utils::graph::topological_layers;

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
        let token = unsafe { GhostToken::new() };
        let mut errors = Vec::new();

        context.tree().dfs().iter(&token, |node, side| {
            match analyzers
                .iter()
                .try_for_each(|a| a.analyze(node, side, &token, context))
            {
                Ok(_) => {}
                Err(e) => errors.push(e),
            };
        });
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'c, C: Context<'c>, E> Traversable<'c, C, E> for TraverseSet<'c, C, E> {
    fn traverse(&self, mut context: C) -> Result<C, (Vec<E>, C)> {
        let sorted = topological_layers(&self.inner);
        for layer in sorted {
            let (_transformers, analyzers): (Vec<_>, Vec<_>) = layer
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
