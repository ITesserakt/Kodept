use std::fmt::{Debug, Formatter};
use std::sync::Weak;

use derive_more::{Deref, DerefMut, From};
use qcell::{TLCell, TLCellOwner};

use crate::graph::utils::OptVec;
use crate::graph::GenericASTNode;

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
pub struct Owned<T = GenericASTNode>(CellImpl<T>);

#[derive(Deref, From)]
pub struct BorrowedNode<T = GenericASTNode>(Weak<CellImpl<OwnedNodeImpl<T>>>);
#[derive(Deref, DerefMut, From)]
pub struct GhostToken(CellOwnerImpl);

pub type RefNode<'arena, T = GenericASTNode> = &'arena Owned<T>;

impl<T> Owned<T> {
    pub fn new<U: Into<T>>(data: U) -> Self {
        Self(TLCell::new(data.into()))
    }
}

impl GhostToken {
    pub unsafe fn new() -> Self {
        Self(TLCellOwner::new())
    }
}

impl<T> Clone for BorrowedNode<T> {
    fn clone(&self) -> Self {
        BorrowedNode(self.0.clone())
    }
}

impl<T: Debug> Debug for Owned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedNode").finish_non_exhaustive()
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
