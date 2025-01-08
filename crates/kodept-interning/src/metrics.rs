use crate::{SharedStr, GLOBAL_STRING_POOL, TOTAL_SHARES};
use std::ops::Deref;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub struct InterningMetrics {
    pub total_shares: usize,
    pub total_items: usize,
    pub approximate_memory_savings: f64,
}

struct CollectProperties {
    count: usize,
    total_size: usize,
}

impl<A: Deref<Target = Box<str>>> FromIterator<A> for CollectProperties {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut count = 0;
        let mut total_size = 0;
        for item in iter {
            count += 1;
            total_size += item.deref().len();
        }

        Self { count, total_size }
    }
}

impl InterningMetrics {
    pub fn gather() -> Self {
        let total_shares = TOTAL_SHARES.load(Ordering::Acquire);
        let pooled_entries: CollectProperties = GLOBAL_STRING_POOL.pooled();
        let coefficient = total_shares as f64 / (pooled_entries.count as f64);

        let total_allocated_size_for_indexes = total_shares * size_of::<SharedStr>();
        let total_allocated_size_for_strings =
            pooled_entries.count * size_of::<Box<str>>() + pooled_entries.total_size;

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
