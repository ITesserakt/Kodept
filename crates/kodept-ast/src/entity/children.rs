use crate::syntax_tree::children::arity::{Optional, Plural, Singular};
use smallvec::SmallVec;

pub trait Arity {
    type Container<T>;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::Container<T>;
}

impl Arity for Singular {
    type Container<T> = T;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::Container<T> {
        match iter.into_iter().next() {
            None => panic!("Node must have exactly one child"),
            Some(x) => x,
        }
    }
}

impl Arity for Optional {
    type Container<T> = Option<T>;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::Container<T> {
        let mut iter = iter.into_iter();
        let first = iter.next();
        let second = iter.next();
        match (first, second) {
            (Some(x), None) => Some(x),
            (None, None) => None,
            (Some(_), Some(_)) => panic!("Node must have at most one child"),
            (None, Some(_)) => unreachable!("Iterator is not fused"),
        }
    }
}

impl Arity for Plural {
    type Container<T> = SmallVec<[T; 8]>;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::Container<T> {
        Self::Container::from_iter(iter)
    }
}
