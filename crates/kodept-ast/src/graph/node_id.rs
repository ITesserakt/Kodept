use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use derive_more::{Display, From};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(From, Display)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
#[display(fmt = "{_0}")]
pub struct NodeId<Node: ?Sized>(usize, PhantomData<Node>);

impl<T> NodeId<T> {
    #[deprecated]
    pub fn next<U>(&self) -> NodeId<U> {
        NodeId(self.0 + 1, PhantomData)
    }

    #[inline]
    pub fn cast<U: TryFrom<T>>(self) -> NodeId<U> {
        NodeId(self.0, PhantomData)
    }

    /// # Safety
    /// Caller should carefully do this, because it may violate some contracts
    #[inline]
    #[deprecated]
    pub unsafe fn cast_unchecked<U>(self) -> NodeId<U> {
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
        *self
    }
}

impl<T> Copy for NodeId<T> {}

impl<T> From<NodeId<T>> for usize {
    fn from(value: NodeId<T>) -> Self {
        value.0
    }
}
