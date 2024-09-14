#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub trait HasLength {
    fn len(&self) -> usize;
}

pub trait IntoCommonIter {
    type Item;

    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;

    #[cfg(feature = "parallel")]
    type IntoParIter: ParallelIterator<Item = Self::Item>;
    #[cfg(feature = "parallel")]
    fn into_par_iter(self) -> Self::IntoParIter;
}

#[cfg(feature = "parallel")]
impl<I: IntoIterator<Item = T> + IntoParallelIterator<Item = T>, T> IntoCommonIter for I {
    type Item = T;
    type IntoIter = I::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        I::into_iter(self)
    }

    type IntoParIter = I::Iter;
    fn into_par_iter(self) -> Self::IntoParIter {
        I::into_par_iter(self)
    }
}

#[cfg(not(feature = "parallel"))]
impl<I: IntoIterator<Item = T>, T> IntoCommonIter for I {
    type Item = T;
    type IntoIter = I::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        I::into_iter(self)
    }
}

impl<T> HasLength for [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> HasLength for &[T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> HasLength for Option<T> {
    fn len(&self) -> usize {
        match self {
            None => 0,
            Some(_) => 1,
        }
    }
}

impl<T> HasLength for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T, const N: usize> HasLength for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T> HasLength for &Box<[T]> {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}
