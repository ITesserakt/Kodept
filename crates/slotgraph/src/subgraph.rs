use crate::key::CommonKey;
use crate::parts::{Edge, KeyRef};
use crate::parts::{EdgeKey, NodeKey};
use petgraph::visit::{
    Data, GraphBase, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers, IntoNodeReferences,
    NodeIndexable,
};
use petgraph::EdgeType;
use slotmap::secondary::Iter;
use slotmap::SecondaryMap;
use std::iter::FusedIterator;

pub struct SubGraph<N, E, D: EdgeType> {
    nodes: SecondaryMap<CommonKey, N>,
    edges: SecondaryMap<CommonKey, Edge<E, D>>,
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct NodeIdentifiersIter<'t, N>(Iter<'t, CommonKey, N>);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct NodeReferencesIter<'t, N>(Iter<'t, CommonKey, N>);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct EdgeReferencesIter<'t, E, D: EdgeType>(Iter<'t, CommonKey, Edge<E, D>>);

impl<N, E, D: EdgeType> SubGraph<N, E, D> {
    pub(crate) fn new(
        nodes: SecondaryMap<CommonKey, N>,
        edges: SecondaryMap<CommonKey, Edge<E, D>>,
    ) -> Self {
        Self { nodes, edges }
    }
}

impl<N, E, D: EdgeType> GraphBase for SubGraph<N, E, D> {
    type EdgeId = EdgeKey;
    type NodeId = NodeKey;
}

impl<N, E, D: EdgeType> GraphProp for SubGraph<N, E, D> {
    type EdgeType = Edge<E, D>;
}

impl<N, E, D: EdgeType> Data for SubGraph<N, E, D> {
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<N, E, D: EdgeType> NodeIndexable for SubGraph<N, E, D> {
    fn node_bound(&self) -> usize {
        self.nodes.len()
    }

    fn to_index(&self, a: Self::NodeId) -> usize {
        a.0.to_index() as usize
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        NodeKey(CommonKey::from_index(i as u64))
    }
}

impl<'a, N, E, D: EdgeType> IntoNodeIdentifiers for &'a SubGraph<N, E, D> {
    type NodeIdentifiers = NodeIdentifiersIter<'a, N>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiersIter(self.nodes.iter())
    }
}

impl<'a, N, E, D: EdgeType> IntoNodeReferences for &'a SubGraph<N, E, D> {
    type NodeRef = KeyRef<'a, N>;
    type NodeReferences = NodeReferencesIter<'a, N>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferencesIter(self.nodes.iter())
    }
}

impl<'a, N, E, D: EdgeType> IntoEdgeReferences for &'a SubGraph<N, E, D> {
    type EdgeRef = KeyRef<'a, Edge<E, D>>;
    type EdgeReferences = EdgeReferencesIter<'a, E, D>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferencesIter(self.edges.iter())
    }
}

impl<'t, N> Iterator for NodeIdentifiersIter<'t, N> {
    type Item = NodeKey;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|it| NodeKey(it.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'t, N> Iterator for NodeReferencesIter<'t, N> {
    type Item = KeyRef<'t, N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|it| KeyRef::new(it.0, it.1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'t, E, D: EdgeType> Iterator for EdgeReferencesIter<'t, E, D> {
    type Item = KeyRef<'t, Edge<E, D>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|it| KeyRef::new(it.0, it.1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'t, N> FusedIterator for NodeIdentifiersIter<'t, N> {}
impl<'t, N> FusedIterator for NodeReferencesIter<'t, N> {}
impl<'t, E, D: EdgeType> FusedIterator for EdgeReferencesIter<'t, E, D> {}

impl<'t, N> ExactSizeIterator for NodeIdentifiersIter<'t, N> {}
impl<'t, N> ExactSizeIterator for NodeReferencesIter<'t, N> {}
impl<'t, E, D: EdgeType> ExactSizeIterator for EdgeReferencesIter<'t, E, D> {}
