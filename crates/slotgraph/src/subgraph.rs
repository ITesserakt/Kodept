use crate::key::CommonKey;
use crate::parts::{Edge, KeyRef};
use crate::parts::{EdgeKey, NodeKey};
use petgraph::visit::{
    Data, GraphBase, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers, IntoNodeReferences,
    NodeIndexable,
};
use petgraph::EdgeType;
use slotmap::SecondaryMap;

pub struct SubGraph<N, E, D: EdgeType> {
    nodes: SecondaryMap<CommonKey, N>,
    edges: SecondaryMap<CommonKey, Edge<E, D>>,
}

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
    fn node_bound(self: &Self) -> usize {
        self.nodes.len()
    }

    fn to_index(self: &Self, a: Self::NodeId) -> usize {
        a.0.to_index() as usize
    }

    fn from_index(self: &Self, i: usize) -> Self::NodeId {
        NodeKey(CommonKey::from_index(i as u64))
    }
}

impl<'a, N, E, D: EdgeType> IntoNodeIdentifiers for &'a SubGraph<N, E, D> {
    type NodeIdentifiers = impl Iterator<Item = Self::NodeId>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.nodes.iter().map(|it| NodeKey(it.0))
    }
}

impl<'a, N, E, D: EdgeType> IntoNodeReferences for &'a SubGraph<N, E, D> {
    type NodeRef = KeyRef<'a, N>;
    type NodeReferences = impl Iterator<Item = Self::NodeRef>;

    fn node_references(self) -> Self::NodeReferences {
        self.nodes.iter().map(|(k, v)| KeyRef::new(k, v))
    }
}

impl<'a, N, E, D: EdgeType> IntoEdgeReferences for &'a SubGraph<N, E, D> {
    type EdgeRef = KeyRef<'a, Edge<E, D>>;
    type EdgeReferences = impl Iterator<Item = Self::EdgeRef>;

    fn edge_references(self) -> Self::EdgeReferences {
        self.edges.iter().map(|(k, v)| KeyRef::new(k, v))
    }
}
