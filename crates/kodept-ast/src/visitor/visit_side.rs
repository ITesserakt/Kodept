use derive_more::{From, Into, IsVariant};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use crate::graph::{GenericASTNode, GhostToken, RefMut};

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

#[derive(From, Into)]
pub struct Access<'arena, 'token, N>(&'token mut GhostToken, RefMut<'arena, N>);

pub struct VisitGuard<'arena, 'token, N>(VisitSide, RefMut<'arena, N>, &'token mut GhostToken);

impl<'arena, 'token, N> VisitGuard<'arena, 'token, N> {
    pub fn new(side: VisitSide, access: RefMut<'arena, N>, token: &'token mut GhostToken) -> Self {
        Self(side, access, token)
    }

    pub fn allow_only<E>(self, matches: VisitSide) -> Result<Access<'arena, 'token, N>, Skip<E>> {
        self.0.guard(matches).map(|_| Access(self.2, self.1))
    }

    pub fn allow_all(self) -> (Access<'arena, 'token, N>, VisitSide) {
        (Access(self.2, self.1), self.0)
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

pub trait SkipExt<T, E> {
    fn skipped(self) -> Result<T, E>;
}

impl<T, E> SkipExt<T, E> for Result<T, Skip<E>>
where
    T: Default,
{
    fn skipped(self) -> Result<T, E> {
        match self {
            Ok(_) => Ok(T::default()),
            Err(Skip::SideGuard) => Ok(T::default()),
            Err(Skip::WithError(e)) => Err(e),
        }
    }
}

impl<'arena, 'token, N: 'arena> Deref for Access<'arena, 'token, N>
where
    for<'a> &'a N: TryFrom<&'a GenericASTNode>,
    for<'a> <&'a GenericASTNode as TryInto<&'a N>>::Error: Debug,
{
    type Target = N;

    fn deref(&self) -> &Self::Target {
        self.1.borrow(self.0)
    }
}

impl<'arena, 'token, N: 'arena> DerefMut for Access<'arena, 'token, N>
where
    for<'a> &'a N: TryFrom<&'a GenericASTNode>,
    for<'a> &'a mut N: TryFrom<&'a mut GenericASTNode>,
    for<'a> <&'a GenericASTNode as TryInto<&'a N>>::Error: Debug,
    for<'a> <&'a mut GenericASTNode as TryInto<&'a mut N>>::Error: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1.borrow_mut(self.0)
    }
}

impl<'arena, 'token, N> Access<'arena, 'token, N> {
    pub fn token(&self) -> &GhostToken {
        self.0
    }
    
    pub fn token_mut(&mut self) -> &mut GhostToken {
        self.0
    }
}
