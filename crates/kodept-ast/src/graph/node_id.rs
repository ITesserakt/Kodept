use std::fmt::{Debug, Formatter};

use crate::graph::any_node::{AnyNode, SubEnum};
use derive_more::{Display, From};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use slotgraph::dag::NodeKey;
use slotgraph::Key;

#[derive(Display, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum NodeId<Node> {
    Root,
    Key(Key<Node>),
}

pub type GenericNodeId = NodeId<AnyNode>;
pub type GenericNodeKey = Key<AnyNode>;

impl<Node> From<NodeKey> for NodeId<Node> {
    fn from(value: NodeKey) -> Self {
        match value {
            NodeKey::Root => NodeId::Root,
            NodeKey::Child(k) => NodeId::Key(k.into()),
        }
    }
}

impl<Node> From<slotgraph::NodeKey> for NodeId<Node> {
    fn from(value: slotgraph::NodeKey) -> Self {
        NodeId::Key(value.into())
    }
}

impl<Node> From<NodeId<Node>> for NodeKey {
    fn from(value: NodeId<Node>) -> Self {
        match value {
            NodeId::Root => NodeKey::Root,
            NodeId::Key(k) => k.into(),
        }
    }
}

impl<T> NodeId<T> {
    pub fn null() -> Self {
        Self::Key(Key::null())
    }

    pub fn cast<U>(self) -> NodeId<U>
    where
        U: TryFrom<T> + SubEnum,
    {
        match self {
            NodeId::Root => NodeId::Root,
            NodeId::Key(k) => NodeId::Key(k.coerce()),
        }
    }

    pub fn as_key(&self) -> Option<GenericNodeKey>
    where
        AnyNode: TryFrom<T>,
    {
        match self {
            NodeId::Root => None,
            NodeId::Key(id) => Some(id.coerce()),
        }
    }
}

impl<T: Into<AnyNode>> NodeId<T> {
    pub fn widen(self) -> GenericNodeId {
        match self {
            NodeId::Root => NodeId::Root,
            NodeId::Key(k) => NodeId::Key(k.coerce()),
        }
    }
}

impl GenericNodeId {
    pub fn coerce<U>(self) -> NodeId<U>
    where
        U: SubEnum,
    {
        match self {
            NodeId::Root => NodeId::Root,
            NodeId::Key(k) => NodeId::Key(k.coerce_unchecked()),
        }
    }
}

impl GenericNodeId {
    pub fn narrow<T: TryFrom<AnyNode>>(self) -> NodeId<T> {
        match self {
            NodeId::Root => NodeId::Root,
            NodeId::Key(k) => NodeId::Key(k.coerce()),
        }
    }
}

impl<T> Debug for NodeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeId::Root => write!(f, "root"),
            NodeId::Key(k) => write!(f, "{}", k),
        }
    }
}

impl<T> PartialEq for NodeId<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NodeId::Root, NodeId::Root) => true,
            (NodeId::Root, NodeId::Key(_)) => false,
            (NodeId::Key(_), NodeId::Root) => false,
            (NodeId::Key(k1), NodeId::Key(k2)) => k1 == k2,
        }
    }
}

impl<T> Eq for NodeId<T> {}

impl<T> Clone for NodeId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeId<T> {}
