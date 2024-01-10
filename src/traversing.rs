use std::mem::replace;
use std::panic::{catch_unwind, AssertUnwindSafe};

use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::{DiGraph, NodeIndex};

use kodept_ast::graph::{ChangeSet, GhostToken};
use kodept_macros::erased::Erased;
use kodept_macros::error::compiler_crash::CompilerCrash;
use kodept_macros::error::report::Report;
use kodept_macros::traits::{Context, UnrecoverableError};

use crate::utils::graph::topological_layers;

#[derive(Debug)]
pub struct TraverseSet<'c, C, E>
where
    C: Context<'c>,
{
    inner: DiGraph<Erased<'c, C, E>, ()>,
}

impl<'c, C: Context<'c>, E> Default for TraverseSet<'c, C, E> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

pub trait Traversable<'c, C: Context<'c>, E> {
    fn traverse(&mut self, context: C) -> Result<C, (Vec<E>, C)>;
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
    fn run_analyzers(&mut self, context: &mut C, analyzers: Vec<NodeIndex>) -> Result<(), Vec<E>> {
        let mut token = unsafe { GhostToken::new() };
        let mut errors = Vec::new();

        context.tree().dfs().iter(|node, side| {
            match analyzers.iter().try_for_each(|&id| {
                let Erased::Analyzer(a) = &mut self.inner[id] else {
                    unreachable!()
                };
                a.analyze(node, side, &mut token, context)
            }) {
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

    fn run_transformers(
        &mut self,
        context: &mut C,
        transformers: Vec<NodeIndex>,
    ) -> Result<ChangeSet, E> {
        let mut token = unsafe { GhostToken::new() };
        let mut changes = ChangeSet::empty();
        let mut error = None;

        context.tree().dfs().iter(|node, side| {
            let mut local_changes = replace(&mut changes, ChangeSet::empty());
            for &t_id in &transformers {
                let Erased::Transformer(t) = &mut self.inner[t_id] else {
                    unreachable!()
                };
                match t.transform(node, side, &mut token, context) {
                    Ok(next_changes) => local_changes = next_changes.merge(local_changes),
                    Err(e) => {
                        error = Some(e);
                        return;
                    }
                }
            }
            changes = local_changes;
        });
        match error {
            None => Ok(changes),
            Some(e) => Err(e),
        }
    }
}

impl<'c, C: Context<'c>> Traversable<'c, C, UnrecoverableError>
    for TraverseSet<'c, C, UnrecoverableError>
{
    fn traverse(&mut self, mut context: C) -> Result<C, (Vec<UnrecoverableError>, C)> {
        let sorted = topological_layers(&self.inner);
        for layer in sorted {
            let (transformers, analyzers) = layer
                .into_iter()
                .partition(|id| matches!(&self.inner[*id], Erased::Transformer(_)));

            let exec_result = catch_unwind(AssertUnwindSafe(|| {
                (
                    self.run_analyzers(&mut context, analyzers),
                    self.run_transformers(&mut context, transformers),
                )
            }));
            match exec_result {
                Ok((Ok(_), Ok(_))) => {}
                Ok((Err(e), Ok(_))) => return Err((e, context)),
                Ok((Ok(_), Err(e))) => return Err((vec![e], context)),
                Ok((Err(mut e1), Err(e2))) => {
                    e1.push(e2);
                    return Err((e1, context));
                }
                Err(crash) => {
                    let report =
                        Report::new(&context.file_path(), vec![], CompilerCrash::new(crash)).into();
                    return Err((vec![report], context));
                }
            };
        }
        Ok(context)
    }
}
