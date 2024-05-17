use std::fmt::{Debug, Formatter};

use derive_more::{Deref, DerefMut, From};
use qcell::{TLCell, TLCellOwner};

use crate::graph::GenericASTNode;

type CellImpl<T> = TLCell<Ghost, T>;
type CellOwnerImpl = TLCellOwner<Ghost>;

#[derive(Debug)]
pub struct Ghost;

#[derive(Deref, From)]
#[repr(transparent)]
pub struct Inaccessible<T = GenericASTNode>(CellImpl<T>);

#[derive(Deref, DerefMut, From)]
pub struct PermTkn(CellOwnerImpl);

pub type RefNode<'arena, T = GenericASTNode> = &'arena Inaccessible<T>;

impl<T> Inaccessible<T> {
    pub fn new<U: Into<T>>(data: U) -> Self {
        Self(TLCell::new(data.into()))
    }
}

impl Default for PermTkn {
    /// Value of this type should be a singleton in one thread
    /// If this contract violated, function will panic
    fn default() -> Self {
        Self(TLCellOwner::new())
    }
}

impl PermTkn {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Debug> Debug for Inaccessible<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inaccessible").finish_non_exhaustive()
    }
}

impl Debug for PermTkn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermissionToken").finish_non_exhaustive()
    }
}
