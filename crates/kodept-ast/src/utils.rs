pub trait HasLength {
    fn len(&self) -> usize;
}

pub trait IntoCommonIter {
    type Item;
    #[cfg(feature = "parallel")]
    type IntoParIter: rayon::prelude::ParallelIterator<Item = Self::Item>;
    type IntoIter: Iterator<Item = Self::Item>;

    #[cfg(feature = "parallel")]
    fn into_par_iter(self) -> Self::IntoParIter;
    fn into_iter(self) -> Self::IntoIter;
}

#[cfg(not(feature = "parallel"))]
impl<I: IntoIterator> IntoCommonIter for I {
    type Item = I::Item;
    type IntoIter = I::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        <I as IntoIterator>::into_iter(self)
    }
}

#[cfg(feature = "parallel")]
impl<I, T> IntoCommonIter for I
where
    I: rayon::prelude::IntoParallelIterator<Item = T>,
    I: IntoIterator<Item = T>,
{
    type Item = T;
    type IntoParIter = I::Iter;
    type IntoIter = I::IntoIter;

    fn into_par_iter(self) -> Self::IntoParIter {
        <I as rayon::prelude::IntoParallelIterator>::into_par_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        <I as IntoIterator>::into_iter(self)
    }
}

impl<T: HasLength + ?Sized> HasLength for &Box<T> {
    #[inline]
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T> HasLength for [T] {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T, const N: usize> HasLength for [T; N] {
    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
}

impl<T> HasLength for &[T] {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> HasLength for Option<T> {
    #[inline]
    fn len(&self) -> usize {
        self.as_ref().map_or(0, |_| 1)
    }
}
