use crate::{GlobalInterner, Internable, Interner};

#[derive(Debug)]
pub struct InterningMetrics {
    pub total_shares: usize,
    pub total_items: usize,
    pub approximate_memory_savings: f64,
}

struct CollectProperties {
    count: usize,
    total_length: usize,
}

pub trait HasLength {
    fn len(&self) -> usize;
}

impl<T> HasLength for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T: HasLength + ?Sized> HasLength for Box<T> {
    fn len(&self) -> usize {
        T::len(self)
    }
}

impl<T: HasLength + ?Sized> HasLength for &T {
    fn len(&self) -> usize {
        T::len(*self)
    }
}

impl HasLength for str {
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl<T> HasLength for [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<A: HasLength> FromIterator<A> for CollectProperties {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut count = 0;
        let mut total_size = 0;
        for item in iter {
            count += 1;
            total_size += item.len();
        }

        Self { count, total_length: total_size }
    }
}

impl InterningMetrics {
    pub fn gather<T: GlobalInterner + ?Sized + 'static + HasLength>() -> Self
    {
        Self::gather_for_interner(T::interner())
    }

    pub fn gather_for_interner<T: Internable + ?Sized + HasLength>(interner: &Interner<T>) -> Self
    {
        let total_shares = interner.total_shares();
        let lock = interner.entries();
        let pooled_entries: CollectProperties = lock.iter().copied().collect();
        let coefficient = total_shares as f64 / (pooled_entries.count as f64);

        let total_allocated_size_for_indexes = total_shares * size_of::<&'static T>();
        let total_allocated_size_for_strings =
            pooled_entries.count * size_of::<Box<T>>() + pooled_entries.total_length;

        Self {
            total_shares,
            total_items: pooled_entries.count,
            approximate_memory_savings: total_allocated_size_for_strings as f64 * coefficient
                - total_allocated_size_for_indexes as f64,
        }
    }

    pub fn sharing_factor(&self) -> f64 {
        self.total_shares as f64 / self.total_items as f64
    }

    pub fn memory_save(&self) -> (f64, &'static str) {
        const SUFFIXES: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut value = self.approximate_memory_savings;

        let mut index = 0;
        loop {
            if value > -1024.0 && value < 1024.0 || index > 5 {
                return (value, SUFFIXES[index]);
            } else {
                value /= 1024.0;
                index += 1;
            }
        }
    }
}
