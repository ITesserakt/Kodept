use derive_more::{Deref, From, IsVariant};

use crate::graph::GhostToken;
use crate::visitor::TraversingResult;

#[derive(Debug)]
pub enum Skip<E> {
    WithError(E),
    SideGuard,
}

#[derive(IsVariant, Clone, Ord, PartialOrd, Eq, PartialEq, Copy)]
#[repr(u8)]
pub enum VisitSide {
    Entering,
    Exiting,
    Leaf,
}

#[derive(From, Deref)]
#[deref(forward)]
pub struct MutAccess<'token>(&'token mut GhostToken);

#[derive(From, Deref)]
#[deref(forward)]
pub struct RefAccess<'token>(&'token GhostToken);

pub struct VisitGuard<N, Access>(N, VisitSide, Access);

pub type RefVisitGuard<'token, N> = VisitGuard<N, RefAccess<'token>>;
pub type MutVisitGuard<'token, N> = VisitGuard<N, MutAccess<'token>>;

impl<N, T> VisitGuard<N, T> {
    pub fn new<U: Into<T>>(node: N, side: VisitSide, access: U) -> Self {
        Self(node, side, access.into())
    }

    pub fn allow_only<E>(self, matches: VisitSide) -> Result<(N, T), Skip<E>> {
        self.1.guard(matches).map(|_| (self.0, self.2))
    }

    pub fn allow_all(self) -> (N, T, VisitSide) {
        (self.0, self.2, self.1)
    }
}

impl<'token, N> VisitGuard<N, MutAccess<'token>> {
    pub fn access(&mut self) -> &mut GhostToken {
        self.2 .0
    }
}

impl<'token, N> VisitGuard<N, RefAccess<'token>> {
    pub fn access(&self) -> &GhostToken {
        self.2 .0
    }
}

impl VisitSide {
    pub fn guard<E>(self, guarded: VisitSide) -> Result<(), Skip<E>> {
        if self != guarded {
            Err(Skip::SideGuard)
        } else {
            Ok(())
        }
    }
}

impl<E> From<E> for Skip<E> {
    fn from(value: E) -> Self {
        Skip::WithError(value)
    }
}

pub trait SkipExt<E> {
    fn skipped(self) -> Result<(), E>;
}

impl<E> SkipExt<E> for TraversingResult<E> {
    fn skipped(self) -> Result<(), E> {
        match self {
            Ok(_) => Ok(()),
            Err(Skip::SideGuard) => Ok(()),
            Err(Skip::WithError(e)) => Err(e),
        }
    }
}
