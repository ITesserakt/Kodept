use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::replace;
use std::panic::{AssertUnwindSafe, catch_unwind};

use itertools::Itertools;
use petgraph::prelude::{NodeIndex, StableDiGraph};

use kodept_ast::graph::{ChangeSet, GhostToken};
use kodept_macros::erased::{BoxedMacro, CanErase};
use kodept_macros::error::compiler_crash::CompilerCrash;
use kodept_macros::error::report::Report;
use kodept_macros::traits::{Context, UnrecoverableError};

use crate::utils::graph::roots;

type Graph<C, E> = StableDiGraph<BoxedMacro<C, E>, ()>;
type Handler<C, E> = Box<dyn FnOnce(&mut TraverseSet<C, E>, BoxedMacro<C, E>)>;

pub struct TraverseSet<C: Context, E> {
    inner: Graph<C, E>,
    handlers: HashMap<NodeIndex, Handler<C, E>>,
}

pub struct DependencyScope<'a, C: Context, E, T> {
    graph: &'a mut TraverseSet<C, E>,
    self_id: NodeIndex,
    post_handler: Option<Handler<C, E>>,
    _phantom: PhantomData<T>,
}

impl<'a, C: Context, E, T: 'static> DependencyScope<'a, C, E, T> {
    pub fn then<F>(&mut self, handle: F)
    where
        F: FnOnce(&mut TraverseSet<C, E>, T) + 'static,
    {
        self.post_handler = Some(Box::new(|this, obj| {
            let obj = obj.into_any().downcast().expect("Cannot cast");
            handle(this, *obj)
        }))
    }
}

impl<C: Context, E> Default for TraverseSet<C, E> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            handlers: Default::default(),
        }
    }
}

impl<C: Context, E, T> Drop for DependencyScope<'_, C, E, T> {
    fn drop(&mut self) {
        if let Some(handler) = self.post_handler.take() {
            self.graph.handlers.insert(self.self_id, handler);
        }
    }
}

impl<C: Context, E> TraverseSet<C, E> {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn add<T: CanErase<C, Error = E>>(&mut self, item: T) -> DependencyScope<C, E, T> {
        let self_id = self.inner.add_node(item.erase());

        DependencyScope {
            graph: self,
            self_id,
            post_handler: None,
            _phantom: PhantomData,
        }
    }

    fn run_transformers(
        context: &mut C,
        mut transformers: Vec<&mut BoxedMacro<C, E>>,
    ) -> Result<ChangeSet, E> {
        let mut token = unsafe { GhostToken::new() };
        let mut changes = ChangeSet::empty();
        let mut error = None;

        context.tree().dfs().iter(|node, side| {
            let mut local_changes = replace(&mut changes, ChangeSet::empty());
            for t in &mut transformers {
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

impl<C: Context> TraverseSet<C, UnrecoverableError> {
    pub fn traverse(&mut self, mut context: C) -> Result<C, (UnrecoverableError, C)> {
        while self.inner.node_count() != 0 {
            let root_ids = roots(&self.inner).collect_vec();
            let mut layer = root_ids
                .into_iter()
                .filter_map(|id| Some((self.inner.remove_node(id)?, id)))
                .collect_vec();
            let transformers = layer.iter_mut().map(|it| &mut it.0).collect();

            let exec_result = catch_unwind(AssertUnwindSafe(|| {
                Self::run_transformers(&mut context, transformers)
            }));
            for (erased, id) in layer {
                let Some(handler) = self.handlers.remove(&id) else {
                    continue;
                };
                handler(self, erased)
            }
            match exec_result {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => return Err((e, context)),
                Err(crash) => {
                    let report =
                        Report::new(&context.file_path(), vec![], CompilerCrash::new(crash)).into();
                    return Err((report, context));
                }
            };
        }
        Ok(context)
    }
}
