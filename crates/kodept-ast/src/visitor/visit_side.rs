use crate::visitor::TraversingResult;
use derive_more::{Constructor, IsVariant};

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

#[derive(Constructor)]
pub struct VisitGuard<N>(N, VisitSide);

impl<N> VisitGuard<N> {
    pub fn allow_only<E>(self, matches: VisitSide) -> Result<N, Skip<E>> {
        self.1.guard(matches).map(|_| self.0)
    }

    pub fn allow_all(self) -> (N, VisitSide) {
        (self.0, self.1)
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
