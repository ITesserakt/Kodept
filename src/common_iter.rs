pub trait CommonIter {
    type Item;

    fn try_foreach_with<T, E, F>(self, with: T, f: F) -> Result<(), E>
    where
        T: Send + Clone,
        E: Send,
        F: Fn(&mut T, Self::Item) -> Result<(), E>,
        F: Send + Sync;

    fn panic_fuse(self) -> impl CommonIter<Item = Self::Item>;
}

#[cfg(not(feature = "parallel"))]
impl<I: Iterator> CommonIter for I {
    type Item = I::Item;

    fn try_foreach_with<T, E, F>(self, mut with: T, f: F) -> Result<(), E>
    where
        F: Fn(&mut T, Self::Item) -> Result<(), E>,
    {
        for item in self {
            f(&mut with, item)?;
        }
        Ok(())
    }

    fn panic_fuse(self) -> impl CommonIter<Item = Self::Item> {
        self
    }
}

#[cfg(feature = "parallel")]
impl<I: rayon::prelude::ParallelIterator> CommonIter for I {
    type Item = I::Item;

    fn try_foreach_with<T, E, F>(self, with: T, f: F) -> Result<(), E>
    where
        T: Send + Clone,
        E: Send,
        F: Fn(&mut T, Self::Item) -> Result<(), E>,
        F: Send + Sync,
    {
        rayon::prelude::ParallelIterator::try_for_each_with(self, with, f)
    }

    fn panic_fuse(self) -> impl CommonIter<Item = Self::Item> {
        rayon::prelude::ParallelIterator::panic_fuse(self)
    }
}
