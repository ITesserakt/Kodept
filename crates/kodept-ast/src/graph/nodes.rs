use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use derive_more::{Deref, DerefMut, From};
use qcell::{TLCell, TLCellOwner};

use crate::graph::GenericASTNode;
use crate::graph::utils::OptVec;

type CellImpl<T> = TLCell<Ghost, T>;
type CellOwnerImpl = TLCellOwner<Ghost>;

#[derive(Debug)]
pub struct Ghost;

pub struct OwnedNodeImpl<T> {
    pub data: T,
    pub uid: usize,
    pub parent: Option<BorrowedNode<T>>,
    pub edges: OptVec<BorrowedNode<T>>,
}

#[derive(Deref, From)]
#[deref(forward)]
pub struct OwnedNode<T = GenericASTNode>(Arc<CellImpl<OwnedNodeImpl<T>>>);

#[derive(Deref, From)]
pub struct BorrowedNode<T = GenericASTNode>(Weak<CellImpl<OwnedNodeImpl<T>>>);
#[derive(Deref, DerefMut, From)]
pub struct GhostToken(CellOwnerImpl);

#[derive(Deref)]
pub struct RefOwnedNode<'arena, T = GenericASTNode> {
    #[deref]
    arc: Arc<CellImpl<OwnedNodeImpl<T>>>,
    _phantom: PhantomData<&'arena ()>,
}

pub type RefNode<'arena, T = GenericASTNode> = &'arena CellImpl<OwnedNodeImpl<T>>;

impl<T> OwnedNode<T> {
    pub fn new(data: T, uid: usize) -> Self {
        Self(Arc::new(TLCell::new(OwnedNodeImpl {
            data,
            uid,
            parent: None,
            edges: OptVec::Empty,
        })))
    }

    pub fn with_parent(data: T, uid: usize, parent: BorrowedNode<T>) -> Self {
        Self(Arc::new(TLCell::new(OwnedNodeImpl {
            data,
            uid,
            parent: Some(parent),
            edges: OptVec::Empty,
        })))
    }

    pub fn share(&self) -> BorrowedNode<T> {
        BorrowedNode(Arc::downgrade(&self.0))
    }

    pub fn data<'a>(&'a self, token: &'a GhostToken) -> &T {
        &self.0.ro(token).data
    }

    pub fn edges<'a>(&'a self, token: &'a GhostToken) -> &OptVec<BorrowedNode<T>> {
        &self.0.ro(token).edges
    }

    pub fn id(&self, token: &GhostToken) -> usize {
        self.0.ro(token).uid
    }
}

impl GhostToken {
    pub fn new() -> Self {
        Self(TLCellOwner::new())
    }
}

impl<T> Clone for OwnedNode<T> {
    fn clone(&self) -> Self {
        OwnedNode(self.0.clone())
    }
}

impl<T> Clone for BorrowedNode<T> {
    fn clone(&self) -> Self {
        BorrowedNode(self.0.clone())
    }
}

impl<T: Debug> Debug for OwnedNode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedNode")
            .field("strong_count", &Arc::strong_count(&self.0))
            .field("weak_count", &Arc::weak_count(&self.0))
            .finish_non_exhaustive()
    }
}

impl<T: Debug> Debug for BorrowedNode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RcNode")
            .field("strong_count", &Weak::strong_count(&self.0))
            .field("weak_count", &Weak::weak_count(&self.0))
            .finish_non_exhaustive()
    }
}

impl Debug for GhostToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GhostToken").finish_non_exhaustive()
    }
}