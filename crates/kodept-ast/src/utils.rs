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

    #[inline]
    fn into_par_iter(self) -> Self::IntoParIter {
        <I as rayon::prelude::IntoParallelIterator>::into_par_iter(self)
    }

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        <I as IntoIterator>::into_iter(self)
    }
}
