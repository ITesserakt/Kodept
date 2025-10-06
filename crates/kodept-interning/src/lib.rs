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
    
    fn intern(&self) -> Interned<Self> {
        Self::interner().intern(self)
    }
    
    fn intern_owned(self) -> Interned<Self> where 
        Self:Sized {
        Self::interner().intern(&self)
    }
}

pub trait InternInto<T> {
    fn intern_into(self) -> Interned<T>;
}

impl<T: GlobalInterner, U: Into<T>> InternInto<T> for U {
    fn intern_into(self) -> Interned<T> {
        self.into().intern()
    }
}

static GLOBAL_STRING_POOL: Interner<str> = Interner::new();

impl GlobalInterner for str {
    fn interner() -> &'static Interner<Self> {
        &GLOBAL_STRING_POOL
    }
}
