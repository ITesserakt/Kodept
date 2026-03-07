use std::fmt::{Display, Formatter};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A: Display, B: Display> Display for Either<A, B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Either::Left(x) => Display::fmt(x, f),
            Either::Right(x) => Display::fmt(x, f),
        }
    }
}
