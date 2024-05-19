use std::ops::{Deref, DerefMut};

use derive_more::{From, Into, IsVariant};

use kodept_core::{ConvertibleToMut, ConvertibleToRef};

use crate::graph::{GenericASTNode, PermTkn, RefMut};

#[derive(IsVariant, Clone, Ord, PartialOrd, Eq, PartialEq, Copy, Debug)]
#[repr(u8)]
pub enum VisitSide {
    Entering,
    Exiting,
    Leaf,
}

#[derive(From, Into)]
pub struct Access<'arena, 'token, N>(&'token mut PermTkn, RefMut<'arena, N>);

pub struct VisitGuard<'arena, 'token, N>(VisitSide, RefMut<'arena, N>, &'token mut PermTkn);

impl<'arena, 'token, N> VisitGuard<'arena, 'token, N> {
    pub fn new(side: VisitSide, access: RefMut<'arena, N>, token: &'token mut PermTkn) -> Self {
        Self(side, access, token)
    }

    pub fn allow_only(self, matches: VisitSide) -> Option<Access<'arena, 'token, N>> {
        self.0.guard(matches)?;
        Some(Access(self.2, self.1))
    }

    pub fn allow_all(self) -> (Access<'arena, 'token, N>, VisitSide) {
        (Access(self.2, self.1), self.0)
    }
}

impl VisitSide {
    pub fn guard(self, guarded: VisitSide) -> Option<()> {
        (self != guarded).then_some(())
    }
}

impl<'arena, 'token, N: 'arena> Deref for Access<'arena, 'token, N>
where
    GenericASTNode: ConvertibleToRef<N>,
{
    type Target = N;

    fn deref(&self) -> &Self::Target {
        self.1.borrow(self.0)
    }
}

impl<'arena, 'token, N: 'arena> DerefMut for Access<'arena, 'token, N>
where
    GenericASTNode: ConvertibleToMut<N>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.1.borrow_mut(self.0)
    }
}

impl<'arena, 'token, N> Access<'arena, 'token, N> {
    pub fn token(&self) -> &PermTkn {
        self.0
    }

    pub fn token_mut(&mut self) -> &mut PermTkn {
        self.0
    }
}
