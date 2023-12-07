#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use std::hash::{Hash, Hasher};

use derive_more::From;
use std::marker::PhantomData;

#[derive(From, Debug)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
pub struct NodeId<Node: 'static>(u32, PhantomData<Node>);

impl<T: 'static> NodeId<T> {
    pub fn next<U: 'static>(&self) -> NodeId<U> {
        NodeId(self.0 + 1, PhantomData)
    }

    pub(crate) fn cast<U: 'static>(self) -> NodeId<U> {
        NodeId(self.0, PhantomData)
    }
}

impl<T: 'static> NodeId<T> {
    pub fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }
}

impl<T: 'static> Hash for NodeId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: 'static> PartialEq for NodeId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: 'static> Eq for NodeId<T> {}

impl<T: 'static> Clone for NodeId<T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}
