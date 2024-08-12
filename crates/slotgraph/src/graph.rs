use petgraph::visit::{
    Data, EdgeCount, GraphBase, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers,
    IntoNodeReferences, NodeCount,
};
use petgraph::{Directed, Direction, EdgeType};
use slotmap::basic::Iter;
use slotmap::SlotMap;
use std::collections::HashSet;
use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};

use crate::key::CommonKey;
use crate::parts::{Edge, EdgeKey, KeyRef, NodeKey};
use crate::subgraph::SubGraph;

#[derive(Debug)]
pub struct Graph<N, E = (), D: EdgeType = Directed> {
    nodes: SlotMap<CommonKey, N>,
    edges: SlotMap<CommonKey, Edge<E, D>>,
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

impl<N, E, D: EdgeType> Default for Graph<N, E, D> {
    fn default() -> Self {
        Self {
            nodes: SlotMap::default(),
            edges: SlotMap::default(),
        }
    }
}

impl<N, E, D: EdgeType> Graph<N, E, D> {
    pub fn map<M, F>(
        &self,
        node_map: impl Fn(NodeKey, &N) -> M,
        edge_map: impl Fn(EdgeKey, &E) -> F,
    ) -> SubGraph<M, F, D> {
        let sub_node_map = self
            .nodes
            .iter()
            .map(|(k, v)| (k, node_map(NodeKey(k), v)))
            .collect();
        let sub_edge_map = self
            .edges
            .iter()
            .map(|(k, v)| {
                let value = edge_map(EdgeKey(k), &v.data);
                (k, Edge::new(v.from, v.to, value))
            })
            .collect();

        SubGraph::new(sub_node_map, sub_edge_map)
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, data: N) -> NodeKey {
        NodeKey(self.nodes.insert(data))
    }

    pub fn add_node_with_key(&mut self, f: impl FnOnce(NodeKey) -> N) -> NodeKey {
        NodeKey(self.nodes.insert_with_key(|it| f(NodeKey(it))))
    }

    pub fn add_edge(&mut self, from: NodeKey, to: NodeKey, weight: E) -> EdgeKey {
        EdgeKey(self.edges.insert(Edge::new(from, to, weight)))
    }

    pub fn remove_node(&mut self, id: NodeKey) -> Option<N> {
        self.edges
            .retain(move |_, edge| edge.to == id || edge.from == id);
        self.nodes.remove(id.0)
    }

    pub fn remove_edge(&mut self, id: EdgeKey) -> Option<Edge<E, D>> {
        self.edges.remove(id.0)
    }

    pub fn add_child(&mut self, parent: NodeKey, weight: E, data: N) -> (EdgeKey, NodeKey) {
        let child_id = self.add_node(data);
        let edge_id = self.add_edge(parent, child_id, weight);
        (edge_id, child_id)
    }

    pub fn node_weight_mut(&mut self, id: NodeKey) -> Option<&mut N> {
        self.nodes.get_mut(id.0)
    }

    pub fn node_weight(&self, id: NodeKey) -> Option<&N> {
        self.nodes.get(id.0)
    }

    pub fn children(
        &self,
        parent_id: NodeKey,
    ) -> impl FusedIterator<Item = (EdgeKey, NodeKey)> + '_ {
        self.edges.iter().filter_map(move |(k, v)| {
            match (D::is_directed(), v.from == parent_id, v.to == parent_id) {
                (_, true, _) => Some((EdgeKey(k), v.to)),
                (false, false, true) => Some((EdgeKey(k), v.from)),
                (true, false, _) => None,
                (false, false, false) => None,
            }
        })
    }

    pub fn parents(&self, node_id: NodeKey) -> impl Iterator<Item = (EdgeKey, NodeKey)> + '_ {
        self.edges.iter().filter_map(move |(k, v)| {
            match (D::is_directed(), v.from == node_id, v.to == node_id) {
                (_, _, true) => Some((EdgeKey(k), v.from)),
                (false, true, false) => Some((EdgeKey(k), v.from)),
                (true, _, false) => None,
                (false, false, false) => None,
            }
        })
    }
}

impl<N, E, D: EdgeType> Graph<N, E, D> {
    pub fn externals(&self, direction: Direction) -> impl Iterator<Item = NodeKey> + '_ {
        struct Externals<'a, N> {
            internals: HashSet<NodeKey>,
            nodes: Iter<'a, CommonKey, N>,
        }

        impl<'a, N> Iterator for Externals<'a, N> {
            type Item = NodeKey;

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let next = self.nodes.next()?;
                    let key = NodeKey(next.0);
                    if self.internals.contains(&key) {
                        continue;
                    } else {
                        return Some(key);
                    }
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.nodes.size_hint()
            }
        }

        impl<'a, N> FusedIterator for Externals<'a, N> {}

        Externals {
            internals: self
                .edges
                .iter()
                .map(|it| {
                    if direction == Direction::Incoming {
                        it.1.to
                    } else {
                        it.1.from
                    }
                })
                .collect(),
            nodes: self.nodes.iter(),
        }
    }
}

impl<N, E, D: EdgeType> GraphBase for Graph<N, E, D> {
    type EdgeId = EdgeKey;
    type NodeId = NodeKey;
}

impl<N, E, D: EdgeType> GraphProp for Graph<N, E, D> {
    type EdgeType = Edge<E, D>;
}

impl<N, E, D: EdgeType> NodeCount for Graph<N, E, D> {
    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl<N, E, D: EdgeType> Data for Graph<N, E, D> {
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<N, E, D: EdgeType> EdgeCount for Graph<N, E, D> {
    fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl<'t, N, E, D: EdgeType> IntoNodeIdentifiers for &'t Graph<N, E, D> {
    type NodeIdentifiers = NodeIdentifiersIter<'t, N>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiersIter(self.nodes.iter())
    }
}

impl<'a, N, E, D: EdgeType> IntoNodeReferences for &'a Graph<N, E, D> {
    type NodeRef = KeyRef<'a, N>;
    type NodeReferences = NodeReferencesIter<'a, N>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferencesIter(self.nodes.iter())
    }
}

impl<'a, N, E, D: EdgeType> IntoEdgeReferences for &'a Graph<N, E, D> {
    type EdgeRef = KeyRef<'a, Edge<E, D>>;
    type EdgeReferences = EdgeReferencesIter<'a, E, D>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferencesIter(self.edges.iter())
    }
}

impl<N, E, D: EdgeType> Index<NodeKey> for Graph<N, E, D> {
    type Output = N;

    fn index(&self, index: NodeKey) -> &Self::Output {
        self.nodes.index(index.0)
    }
}

impl<N, E, D: EdgeType> Index<EdgeKey> for Graph<N, E, D> {
    type Output = Edge<E, D>;

    fn index(&self, index: EdgeKey) -> &Self::Output {
        self.edges.index(index.0)
    }
}

impl<N, E, D: EdgeType> IndexMut<NodeKey> for Graph<N, E, D> {
    fn index_mut(&mut self, index: NodeKey) -> &mut Self::Output {
        self.nodes.index_mut(index.0)
    }
}

impl<N, E, D: EdgeType> IndexMut<EdgeKey> for Graph<N, E, D> {
    fn index_mut(&mut self, index: EdgeKey) -> &mut Self::Output {
        self.edges.index_mut(index.0)
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
