use std::fmt::{Debug, Formatter};

use derive_more::{Deref, DerefMut, From};
use qcell::{TLCell, TLCellOwner};

use crate::graph::GenericASTNode;

type CellImpl<T> = TLCell<Ghost, T>;
type CellOwnerImpl = TLCellOwner<Ghost>;

#[derive(Debug)]
pub struct Ghost;

#[derive(Deref, From)]
pub struct Owned<T = GenericASTNode>(CellImpl<T>);

#[derive(Deref, DerefMut, From)]
pub struct GhostToken(CellOwnerImpl);

pub type RefNode<'arena, T = GenericASTNode> = &'arena Owned<T>;

impl<T> Owned<T> {
    pub fn new<U: Into<T>>(data: U) -> Self {
        Self(TLCell::new(data.into()))
    }
}

impl GhostToken {
    /// Value of this type should be a singleton in one thread
    /// If this contract violated, function will panic
    pub fn new() -> Self {
        Self(TLCellOwner::new())
    }
}

impl<T: Debug> Debug for Owned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedNode").finish_non_exhaustive()
    }
}

impl Debug for GhostToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GhostToken").finish_non_exhaustive()
    }
}
