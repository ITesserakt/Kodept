use std::convert::Infallible;
use derive_more::{Display, Error};
use crate::syntax_tree::children::arity::{Optional, Plural, Singular};
use smallvec::SmallVec;

pub trait Arity {
    type Container<T>;
    type Error;

    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error>;
}

#[derive(Debug, Display, Error)]
pub enum SingleChildError {
    #[display("Expected exactly one child, found zero")]
    Zero,
    #[display("Expected exactly one child, found {_0}")]
    AtLeastTwo(#[error(not(source))] usize),
}

#[derive(Debug, Display, Error)]
pub enum OptionChildError {
    #[display("Expected at most one child, found {_0}")]
    AtLeastTwo(#[error(not(source))] usize),
}

impl Arity for Singular {
    type Container<T> = T;
    type Error = SingleChildError;

    #[inline]
    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error> {
        Optional::try_from_iter(iter)
            .map_err(|OptionChildError::AtLeastTwo(n)| SingleChildError::AtLeastTwo(n))
            .and_then(|it| it.ok_or(SingleChildError::Zero))
    }
}

impl Arity for Optional {
    type Container<T> = Option<T>;

    type Error = OptionChildError;

    #[inline]
    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error> {
        let mut iter = iter.into_iter().fuse();
        let first = iter.next();
        let second = iter.next();
        match (first, second) {
            (None, None) => Ok(None),
            (Some(x), None) => Ok(Some(x)),
            (None, Some(_)) => unreachable!("Iterator is not fused"),
            (Some(_), Some(_)) => Err(OptionChildError::AtLeastTwo(iter.count() + 2)),
        }
    }
}

impl Arity for Plural {
    type Container<T> = SmallVec<[T; 8]>;

    type Error = Infallible;

    #[inline]
    fn try_from_iter<T>(iter: impl IntoIterator<Item=T>) -> Result<Self::Container<T>, Self::Error> {
        Ok(SmallVec::from_iter(iter))
    }
}
