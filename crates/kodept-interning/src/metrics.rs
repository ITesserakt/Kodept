use crate::{GlobalInterner, Internable, Interner};

#[derive(Debug)]
pub struct InterningMetrics {
    pub total_shared_memory: usize,
    pub total_interned_memory: usize,
}

impl InterningMetrics {
    pub fn gather<T: GlobalInterner + ?Sized + 'static>() -> Self {
        Self::gather_for_interner(T::interner())
    }

    pub fn gather_for_interner<T: Internable + ?Sized>(interner: &Interner<T>) -> Self {
        // Haha, some magic constant
        const HASHMAP_ENTRY_OVERHEAD: usize = 32;

        let interned_items = interner.entries();
        let total_shares = interner.total_shares();
        let shared_memory = total_shares * size_of::<&T>() + {
            // assume that approximate size of each shared item is equal
            interned_items
                .iter()
                .next()
                .map_or(0, |&it| size_of_val(it))
        };
        let mut interned_memory = 0;
        for &item in interned_items.iter() {
            // approximate size of itself + overhead per entry + overhead per ref
            interned_memory += size_of_val(item) + HASHMAP_ENTRY_OVERHEAD + size_of::<&T>();
        }

        Self {
            total_shared_memory: shared_memory,
            total_interned_memory: interned_memory,
        }
    }

    pub fn sharing_factor(&self) -> f64 {
        if self.total_shared_memory == 0 {
            0.0
        } else {
            let savings = self.total_shared_memory as f64 - self.total_interned_memory as f64;
            savings / self.total_shared_memory as f64
        }
    }

    pub fn memory_save(&self) -> (f64, &'static str) {
        const SUFFIXES: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut value = self.total_shared_memory as f64 - self.total_interned_memory as f64;

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
