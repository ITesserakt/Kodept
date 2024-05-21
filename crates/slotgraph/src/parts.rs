use std::fmt::{Display, Formatter};
use crate::key::CommonKey;
use crate::Key;
use petgraph::prelude::EdgeRef;
use petgraph::visit::NodeRef;
use petgraph::{Directed, EdgeType};
use std::marker::PhantomData;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
#[repr(transparent)]
pub struct NodeKey(pub(crate) CommonKey);

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
#[repr(transparent)]
pub struct EdgeKey(pub(crate) CommonKey);

pub struct KeyRef<'a, T> {
    pub(crate) data: &'a T,
    pub(crate) key: CommonKey,
}

#[derive(Debug)]
pub struct Edge<T = (), Direction: EdgeType = Directed> {
    pub from: NodeKey,
    pub to: NodeKey,
    pub data: T,
    _phantom: PhantomData<Direction>,
}

impl<T, D: EdgeType> Edge<T, D> {
    pub fn new(from: NodeKey, to: NodeKey, data: T) -> Self {
        Edge {
            from,
            to,
            data,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T> KeyRef<'a, T> {
    pub(crate) fn new(id: CommonKey, value: &'a T) -> Self {
        Self {
            data: value,
            key: id,
        }
    }
}

impl<'a, T> Copy for KeyRef<'a, T> {}

impl<'a, T> Clone for KeyRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> NodeRef for KeyRef<'a, T> {
    type NodeId = NodeKey;
    type Weight = T;

    fn id(&self) -> Self::NodeId {
        NodeKey(self.key)
    }

    fn weight(&self) -> &Self::Weight {
        self.data
    }
}

impl<'a, T, D: EdgeType> EdgeRef for KeyRef<'a, Edge<T, D>> {
    type NodeId = NodeKey;
    type EdgeId = EdgeKey;
    type Weight = T;

    fn source(&self) -> Self::NodeId {
        self.data.from
    }

    fn target(&self) -> Self::NodeId {
        self.data.to
    }

    fn weight(&self) -> &Self::Weight {
        &self.data.data
    }

    fn id(&self) -> Self::EdgeId {
        EdgeKey(self.key)
    }
}

impl<E, T: EdgeType> EdgeType for Edge<E, T> {
    fn is_directed() -> bool {
        T::is_directed()
    }
}

impl<T> From<NodeKey> for Key<T> {
    fn from(value: NodeKey) -> Self {
        value.0.into()
    }
}

impl Display for NodeKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use slotmap::Key;
        write!(f, "{:?}", self.0.data())
    }
}
