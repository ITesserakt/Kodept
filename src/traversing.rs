use crate::traversing::OptionalContext::Defined;
use crate::utils;
use kodept_ast::graph::visitor::ASTVisitor;
use kodept_ast::AST;
use kodept_macros::erased::Erased;
use kodept_macros::traits::Context;
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::{DiGraph, NodeIndex};
use std::cell::RefCell;
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
    fn traverse(&self, ast: &mut AST, context: C) -> Result<C, (E, C)>;
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
}

impl<'c, C: Context<'c>, E> Traversable<'c, C, E> for TraverseSet<'c, C, E> {
    fn traverse(&self, ast: &mut AST, context: C) -> Result<C, (E, C)> {
        let context = RefCell::new(Defined(context));
        let layers = utils::graph::topological_layers(&self.inner);

        for layer in layers {
            let visitor = ASTVisitor::new(ast, |node, side| {
                for item in &layer {
                    let Some(item) = self.inner.node_weight(*item) else {
                        continue;
                    };
                    let mut guard = context.borrow_mut();
                    match item {
                        Erased::Transformer(x) => x.transform(node, side, &mut guard),
                        Erased::Analyzer(x) => x.analyze(node, side, &mut guard),
                    }?;
                }
                Result::<_, E>::Ok(())
            });
            visitor
                .apply()
                .map_err(|e| (e, context.take().into_inner()))?;
        }
        Ok(context.take().into_inner())
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
