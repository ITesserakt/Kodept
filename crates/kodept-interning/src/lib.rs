//! This crate contains a wrapper around string interner.

mod implementation;
pub mod metrics;
mod fixed_hasher;
#[cfg(feature = "code_holder")]
mod code_holder;

#[cfg(feature = "code_holder")]
pub use code_holder::InterningCodeHolder;
pub use implementation::{Interned, Internable, Interner};

pub trait GlobalInterner: Internable {
    fn interner() -> &'static Interner<Self>;
}

static GLOBAL_STRING_POOL: Interner<str> = Interner::new();

impl GlobalInterner for str {
    fn interner() -> &'static Interner<Self> {
        &GLOBAL_STRING_POOL
    }
}
