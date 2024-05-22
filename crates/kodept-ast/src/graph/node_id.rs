use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use derive_more::{Display, From};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use slotgraph::{Key, NodeKey};

use crate::graph::{AnyNode, SubEnum};

#[derive(Display, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
pub struct NodeId<Node>(Key<Node>);

pub type GenericNodeId = NodeId<AnyNode>;
pub type GenericNodeKey = Key<AnyNode>;

impl From<GenericNodeId> for Key<AnyNode> {
    fn from(value: GenericNodeId) -> Self {
        value.0
    }
}

impl<T> From<NodeKey> for NodeId<T> {
    fn from(value: NodeKey) -> Self {
        NodeId(value.into())
    }
}

impl<T> From<NodeId<T>> for NodeKey {
    fn from(value: NodeId<T>) -> Self {
        value.0.into()
    }
}

impl<T> NodeId<T> {
    pub fn null() -> Self {
        Self(Key::null())
    }

    pub fn cast<U>(self) -> NodeId<U>
    where
        U: TryFrom<T> + SubEnum,
    {
        NodeId(self.0.coerce())
    }
}

impl<T: Into<AnyNode>> NodeId<T> {
    pub fn widen(self) -> GenericNodeId {
        NodeId(self.0.coerce())
    }
}

impl GenericNodeId {
    pub fn coerce<U>(self) -> NodeId<U>
    where
        U: SubEnum,
    {
        NodeId(self.0.coerce_unchecked())
    }
}

impl GenericNodeId {
    pub fn narrow<T: TryFrom<AnyNode>>(self) -> NodeId<T> {
        NodeId(self.0.coerce())
    }
}

impl<T> Debug for NodeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Hash for NodeId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> PartialEq for NodeId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for NodeId<T> {}

impl<T> Clone for NodeId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeId<T> {}
