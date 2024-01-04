use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use derive_more::From;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
pub struct NodeId<Node: ?Sized>(usize, PhantomData<Node>);

impl<T> NodeId<T> {
    pub fn next<U>(&self) -> NodeId<U> {
        NodeId(self.0 + 1, PhantomData)
    }

    #[inline]
    pub fn cast<U>(self) -> NodeId<U> {
        NodeId(self.0, PhantomData)
    }
}

impl<T> Debug for NodeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> NodeId<T> {
    pub fn new(id: usize) -> Self {
        Self(id, PhantomData)
    }
}

impl<T> Default for NodeId<T> {
    fn default() -> Self {
        NodeId::new(0)
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
        Self::new(self.0)
    }
}

impl<T> Copy for NodeId<T> {}

impl<T> From<NodeId<T>> for usize {
    fn from(value: NodeId<T>) -> Self {
        value.0
    }
}
