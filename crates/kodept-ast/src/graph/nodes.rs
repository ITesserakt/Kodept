use crate::graph::utils::OptVec;
use derive_more::Deref;
use ghost_cell::GhostCell;
use std::sync::{Arc, Weak};

pub struct OwnedNodeImpl<'id, T> {
    pub data: T,
    pub uid: usize,
    pub parent: Option<RcNode<'id, T>>,
    pub edges: OptVec<RcNode<'id, T>>,
}

#[derive(Deref)]
#[deref(forward)]
pub struct OwnedNode<'id, T>(Arc<GhostCell<'id, OwnedNodeImpl<'id, T>>>);

#[derive(Deref)]
pub struct RcNode<'id, T>(Weak<GhostCell<'id, OwnedNodeImpl<'id, T>>>);

pub type RefNode<'arena, 'id, T> = &'arena GhostCell<'id, OwnedNodeImpl<'id, T>>;

impl<'id, T> OwnedNode<'id, T> {
    pub fn new(data: T, uid: usize) -> Self {
        Self(Arc::new(GhostCell::new(OwnedNodeImpl {
            data,
            uid,
            parent: None,
            edges: OptVec::Empty,
        })))
    }

    pub fn with_parent(mut self, parent: RcNode<'id, T>) -> OwnedNode<'id, T> {
        self.0.get_mut().parent = Some(parent);
        self
    }

    pub fn share(&self) -> RcNode<'id, T> {
        RcNode(Arc::downgrade(&self.0))
    }
}

impl<T> Clone for OwnedNode<'_, T> {
    fn clone(&self) -> Self {
        OwnedNode(self.0.clone())
    }
}

impl<T> Clone for RcNode<'_, T> {
    fn clone(&self) -> Self {
        RcNode(self.0.clone())
    }
}
