use std::collections::HashSet;
use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};

use petgraph::visit::{
    Data, EdgeCount, GraphBase, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers,
    IntoNodeReferences, NodeCount,
};
use petgraph::{Directed, Direction, EdgeType};
use slotmap::SlotMap;

use crate::key::CommonKey;
use crate::parts::{Edge, EdgeKey, KeyRef, NodeKey};
use crate::subgraph::SubGraph;

#[derive(Debug)]
pub struct Graph<N, E = (), D: EdgeType = Directed> {
    nodes: SlotMap<CommonKey, N>,
    edges: SlotMap<CommonKey, Edge<E, D>>,
}

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
        self.edges.retain(move |_, edge| {
            edge.to == id || edge.from == id
        });
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

    pub fn children(&self, parent_id: NodeKey) -> impl FusedIterator<Item = (EdgeKey, NodeKey)> + '_ {
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
            nodes: slotmap::basic::Iter<'a, CommonKey, N>,
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

impl<N, E, D: EdgeType> IntoNodeIdentifiers for &Graph<N, E, D> {
    type NodeIdentifiers = impl Iterator<Item = NodeKey>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.nodes.iter().map(|it| NodeKey(it.0))
    }
}

impl<'a, N, E, D: EdgeType> IntoNodeReferences for &'a Graph<N, E, D> {
    type NodeRef = KeyRef<'a, N>;
    type NodeReferences = impl Iterator<Item = KeyRef<'a, N>>;

    fn node_references(self) -> Self::NodeReferences {
        self.nodes.iter().map(|it| KeyRef::new(it.0, it.1))
    }
}

impl<'a, N, E, D: EdgeType> IntoEdgeReferences for &'a Graph<N, E, D> {
    type EdgeRef = KeyRef<'a, Edge<E, D>>;
    type EdgeReferences = impl Iterator<Item = KeyRef<'a, Edge<E, D>>>;

    fn edge_references(self) -> Self::EdgeReferences {
        self.edges.iter().map(|it| KeyRef::new(it.0, it.1))
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
