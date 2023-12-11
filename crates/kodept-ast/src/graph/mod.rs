#![allow(private_bounds)]

pub mod children;
pub mod generic_node;
pub mod traits;
pub mod visitor;

use crate::graph::children::HasChildrenMarker;
use crate::graph::generic_node::GenericASTNode;
use crate::graph::traits::{Identifiable, PopulateTree};
use crate::node_id::NodeId;
use crate::rlt_accessor::{ASTFamily, RLTFamily};
use crate::traits::Linker;
use kodept_core::structure::span::CodeHolder;
use petgraph::prelude::{EdgeRef, StableGraph};
use petgraph::{Directed, Direction};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::{Context, SizeOf};
use smallvec::SmallVec;

type Graph = StableGraph<GenericASTNode, (), Directed, usize>;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct SyntaxTree {
    inner: Graph,
}

pub struct ChildScope<'a, T: 'static> {
    graph_ref: &'a mut SyntaxTree,
    parent_id: NodeId<T>,
}

impl SyntaxTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node<T>(&mut self, node: T) -> ChildScope<T>
    where
        T: Into<GenericASTNode> + Identifiable,
        for<'a> &'a mut T: TryFrom<&'a mut GenericASTNode>,
    {
        let id: NodeId<T> = self.inner.add_node(node.into()).into();
        {
            let node: Result<&mut T, _> = self.inner.node_weight_mut(id.into()).unwrap().try_into();
            match node {
                Ok(x) => x.set_id(id),
                Err(_) => unreachable!(),
            };
        }
        ChildScope {
            graph_ref: self,
            parent_id: id,
        }
    }

    pub(crate) fn children_of<T, U>(&self, id: NodeId<T>) -> SmallVec<&U, 1>
    where
        for<'a> &'a U: TryFrom<&'a GenericASTNode>,
    {
        self.inner
            .edges_directed(id.into(), Direction::Outgoing)
            .map(|x| x.target())
            .filter_map(|x| (&self.inner[x]).try_into().ok())
            .collect()
    }

    pub(crate) fn children_of_id<T, U>(&mut self, id: NodeId<T>) -> SmallVec<NodeId<U>, 1> {
        self.inner
            .neighbors_directed(id.into(), Direction::Outgoing)
            .map(|x| x.into())
            .collect()
    }

    pub fn get_mut<T>(&mut self, id: NodeId<T>) -> Option<&mut T>
    where
        for<'a> &'a mut T: TryFrom<&'a mut GenericASTNode>,
    {
        self.inner
            .node_weight_mut(id.into())
            .and_then(|x| x.try_into().ok())
    }
}

impl<'a, T: 'static> ChildScope<'a, T> {
    pub fn add_child<U>(&mut self, node: U) -> NodeId<U>
    where
        U: Into<GenericASTNode>,
        T: HasChildrenMarker<U>,
    {
        let id = self.graph_ref.inner.add_node(node.into());
        self.graph_ref.inner.add_edge(self.parent_id.into(), id, ());
        id.into()
    }

    pub fn add_child_by_id<U>(&mut self, child_id: NodeId<U>)
    where
        U: Into<GenericASTNode>,
        T: HasChildrenMarker<U>,
    {
        self.graph_ref
            .inner
            .add_edge(self.parent_id.into(), child_id.into(), ());
    }

    pub fn with_children_from<'b, I, U>(
        mut self,
        iter: I,
        context: &mut (impl Linker<'b> + CodeHolder),
    ) -> ChildScope<'a, T>
    where
        I: IntoIterator<Item = &'b U>,
        U: PopulateTree + 'b,
        <U as PopulateTree>::Output: 'static,
        T: HasChildrenMarker<<U as PopulateTree>::Output>,
    {
        for item in iter {
            let child_id = item.convert(self.graph_ref, context);
            self.add_child_by_id(child_id);
        }
        self
    }

    pub fn with_rlt<'b, U>(self, context: &mut impl Linker<'b>, rlt_node: U) -> ChildScope<'a, T>
    where
        U: Into<RLTFamily<'b>>,
        NodeId<T>: Into<ASTFamily>,
    {
        context.link_ref(self.parent_id, rlt_node);
        self
    }

    pub fn id(&self) -> NodeId<T> {
        self.parent_id
    }
}

impl<'a, T: 'static> From<ChildScope<'a, T>> for NodeId<T> {
    fn from(value: ChildScope<'a, T>) -> Self {
        value.parent_id
    }
}

#[cfg(feature = "size-of")]
impl SizeOf for SyntaxTree {
    fn size_of_children(&self, context: &mut Context) {
        todo!()
    }
}
