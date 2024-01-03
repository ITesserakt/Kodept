#![allow(private_bounds)]

pub mod children;
pub mod generic_node;
mod identity;
pub mod traits;
mod utils;

use crate::graph::children::HasChildrenMarker;
use crate::graph::generic_node::{GenericASTNode, Node, NodeWithParent};
use crate::graph::traits::{Identifiable, PopulateTree};
use crate::graph::utils::{FromOptVec, OptVec};
use crate::node_id::NodeId;
use crate::rlt_accessor::{ASTFamily, RLTFamily};
use crate::traits::Linker;
use crate::visitor::visit_side::VisitSide;
pub use identity::Identity;
use kodept_core::structure::span::CodeHolder;
use petgraph::graph::NodeIndex;
use petgraph::prelude::{EdgeRef, StableGraph};
use petgraph::{Directed, Direction};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::Debug;

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

pub struct SyntaxTreeIter<'a> {
    graph: &'a Graph,
    order: VecDeque<(NodeIndex<usize>, VisitSide)>,
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

    pub(crate) fn children_of<T, U>(&self, id: NodeId<T>) -> OptVec<&U>
    where
        for<'a> &'a U: TryFrom<&'a GenericASTNode>,
        for<'a> <&'a GenericASTNode as TryInto<&'a U>>::Error: Debug,
    {
        self.inner
            .edges_directed(id.into(), Direction::Outgoing)
            .map(|x| x.target())
            .filter_map(|x| (&self.inner[x]).try_into().ok())
            .collect()
    }

    pub(crate) fn children_of_id<T, U, F, A>(&mut self, id: NodeId<T>, mut f: F) -> OptVec<A>
    where
        F: FnMut(&mut U) -> A,
        for<'a> &'a mut U: TryFrom<&'a mut GenericASTNode>,
    {
        let mut neighbours = self
            .inner
            .neighbors_directed(id.into(), Direction::Outgoing)
            .detach();

        let mut result = OptVec::Empty;
        while let Some((_, next_id)) = neighbours.next(&self.inner) {
            let Ok(node) = (&mut self.inner[next_id]).try_into() else {
                continue;
            };
            result.push(f(node));
        }
        result
    }

    pub fn get_mut<T>(&mut self, id: NodeId<T>) -> Option<&mut T>
    where
        for<'a> &'a mut T: TryFrom<&'a mut GenericASTNode>,
    {
        self.inner
            .node_weight_mut(id.into())
            .and_then(|x| x.try_into().ok())
    }

    pub(crate) fn parent_of<T>(&self, id: NodeId<T>) -> &T::Parent
    where
        T: NodeWithParent + Node,
        for<'a> &'a T::Parent: TryFrom<&'a GenericASTNode>,
    {
        let neighbours: OptVec<_> = self
            .inner
            .neighbors_directed(id.into(), Direction::Incoming)
            .map(|x| &self.inner[x])
            .collect();
        let parent = Identity::unwrap(neighbours);
        parent
            .try_into()
            .unwrap_or_else(|_| panic!("Node {:?} has parent with wrong type", id))
    }

    pub(crate) fn parent_of_mut<T>(&mut self, id: NodeId<T>) -> &mut T::Parent
    where
        T: NodeWithParent + Node,
        for<'a> &'a mut T::Parent: TryFrom<&'a mut GenericASTNode>,
    {
        let mut neighbours = self
            .inner
            .neighbors_directed(id.into(), Direction::Incoming);
        let Some(parent_id) = neighbours.next() else {
            panic!("Node should has parent")
        };
        let parent = &mut self.inner[parent_id];
        parent
            .try_into()
            .unwrap_or_else(|_| panic!("Node {:?} has parent with wrong type", id))
    }

    fn dfs<F>(&self, current: NodeIndex<usize>, callback: &mut F)
    where
        F: FnMut(NodeIndex<usize>, VisitSide),
    {
        let children: Vec<_> = self
            .inner
            .neighbors_directed(current, Direction::Outgoing)
            .collect();
        if children.is_empty() {
            callback(current, VisitSide::Leaf)
        } else {
            callback(current, VisitSide::Entering);
            for child in children.into_iter().rev() {
                self.dfs(child, callback);
            }
            callback(current, VisitSide::Exiting);
        }
    }

    pub fn iter(&self) -> SyntaxTreeIter {
        let mut roots = self.inner.externals(Direction::Incoming);
        let Some(root_id) = roots.next() else {
            panic!("AST should always has only one root")
        };
        let mut order = VecDeque::with_capacity(self.inner.node_count());
        self.dfs(root_id, &mut |id, side| order.push_back((id, side)));
        SyntaxTreeIter {
            graph: &self.inner,
            order,
        }
    }

    pub fn iter_mut<F, T>(&mut self, mut handler: F) -> Vec<T>
    where
        F: FnMut(&mut GenericASTNode, VisitSide) -> T,
    {
        let mut roots = self.inner.externals(Direction::Incoming);
        let Some(root_id) = roots.next() else {
            panic!("AST should always has only one root")
        };
        let mut order = VecDeque::with_capacity(self.inner.node_count());
        self.dfs(root_id, &mut |id, side| order.push_back((id, side)));
        order
            .into_iter()
            .map(|(id, side)| handler(&mut self.inner[id], side))
            .collect()
    }
}

impl<'a> Iterator for SyntaxTreeIter<'a> {
    type Item = (&'a GenericASTNode, VisitSide);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, side) = self.order.pop_front()?;
        Some((self.graph.node_weight(id)?, side))
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
