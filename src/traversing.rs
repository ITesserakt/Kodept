use std::collections::HashMap;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};

use itertools::Itertools;
use petgraph::prelude::{NodeIndex, StableDiGraph};

use kodept_ast::graph::{ChangeSet, GhostToken};
use kodept_ast::utils::Execution;
use kodept_macros::erased::{BoxedMacro, CanErase};
use kodept_macros::error::compiler_crash::CompilerCrash;
use kodept_macros::error::report::Report;
use kodept_macros::traits::{Context, MutableContext, UnrecoverableError};

use crate::utils::graph::roots;

type Graph<C, E> = StableDiGraph<BoxedMacro<C, E>, ()>;
type Handler<C, E> = Box<dyn FnOnce(&mut TraverseSet<C, E>, BoxedMacro<C, E>)>;

pub struct TraverseSet<C: Context, E> {
    inner: Graph<C, E>,
    handlers: HashMap<NodeIndex, Handler<C, E>>,
}

pub struct DependencyScope<C: Context, E, T> {
    item: BoxedMacro<C, E>,
    post_handler: Option<Handler<C, E>>,
    _phantom: PhantomData<T>,
}

impl<C: Context, E, T: 'static> DependencyScope<C, E, T> {
    #[must_use]
    pub fn then<F>(mut self, handle: F) -> Self
        where
            F: FnOnce(&mut TraverseSet<C, E>, T) + 'static,
    {
        self.post_handler = Some(Box::new(move |this, obj| {
            let obj = obj.into_any().downcast().expect("Cannot cast");
            handle(this, *obj)
        }));
        self
    }

    pub fn add(self, set: &mut TraverseSet<C, E>) {
        let index = set.inner.add_node(self.item);
        if let Some(handler) = self.post_handler {
            set.handlers.insert(index, handler);
        }
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

impl<C: Context, E> TraverseSet<C, E> {
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn dependency<T: CanErase<C, Error=E>>(&self, item: T) -> DependencyScope<C, E, T> {
        DependencyScope {
            item: item.erase(),
            post_handler: None,
            _phantom: PhantomData,
        }
    }
}

impl<C: MutableContext, E: Into<UnrecoverableError>> TraverseSet<C, E> {
    fn run_macros(
        context: &mut C,
        mut macros: Vec<&mut BoxedMacro<C, E>>,
    ) -> Result<(), UnrecoverableError> {
        let mut token = unsafe { GhostToken::new() };
        let mut changes = ChangeSet::new();

        for (node, side) in context.tree().upgrade().unwrap().dfs() {
            for marc in &mut macros {
                match marc.transform(node, side, &mut token, context) {
                    Execution::Failed(e) => return Err(e.into()),
                    Execution::Completed(next) => changes.extend(next),
                    Execution::Skipped => continue,
                }
            }
        }

        match context.modify_tree(|tree| tree.apply_changes(changes, &mut token)) {
            Ok(_) => Ok(()),
            Err(e) => Err(Report::new(&context.file_path(), vec![], e).into()),
        }
    }
}

impl<C: MutableContext, E: Into<UnrecoverableError>> TraverseSet<C, E> {
    pub fn traverse(&mut self, mut context: C) -> Result<C, (UnrecoverableError, C)> {
        while self.inner.node_count() != 0 {
            let root_ids = roots(&self.inner).collect_vec();
            let mut layer = root_ids
                .into_iter()
                .filter_map(|id| Some((self.inner.remove_node(id)?, id)))
                .collect_vec();
            let macros = layer.iter_mut().map(|it| &mut it.0).collect();

            let exec_result =
                catch_unwind(AssertUnwindSafe(|| Self::run_macros(&mut context, macros)));
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
