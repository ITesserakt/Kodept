
pub trait CommonIter {
    type Item;

    fn try_foreach_with<T, F>(
        self,
        with: T,
        f: F,
    ) -> Option<()>
    where
        T: Send + Clone,
        F: Fn(&mut T, Self::Item) -> Option<()>,
        F: Send + Sync;

    fn panic_fuse(self) -> impl CommonIter<Item = Self::Item>;
}

#[cfg(not(feature = "parallel"))]
impl<I: Iterator> CommonIter for I {
    type Item = I::Item;

    fn try_foreach_with<T, F>(self, mut with: T, f: F) -> Option<()>
    where
        T: Send + Clone,
        F: Fn(&mut T, Self::Item) -> Option<()>,
        F: Send + Sync
    {
        for item in self {
            f(&mut with, item)?;
        }
        Some(())
    }

    fn panic_fuse(self) -> impl CommonIter<Item=Self::Item> {
        self
    }
}

#[cfg(feature = "parallel")]
impl<I: rayon::prelude::ParallelIterator> CommonIter for I {
    type Item = I::Item;

    fn try_foreach_with<T, F>(self, with: T, f: F) -> Option<()>
    where
        T: Send + Clone,
        F: Fn(&mut T, Self::Item) -> Option<()>,
        F: Send + Sync
    {
        rayon::prelude::ParallelIterator::try_for_each_with(self, with, f)
    }

    fn panic_fuse(self) -> impl CommonIter<Item=Self::Item> {
        rayon::prelude::ParallelIterator::panic_fuse(self)
    }
}