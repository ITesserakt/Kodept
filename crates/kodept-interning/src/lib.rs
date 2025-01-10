//! This crate contains a wrapper around string interner.

pub mod metrics;
mod implementation;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::implementation::{Interned, Interner};

static GLOBAL_STRING_POOL: Interner<str> = Interner::new();
static TOTAL_SHARES: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone)]
pub struct InterningCodeHolder<'a, C> {
    inner: C,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, C> InterningCodeHolder<'a, C>
where
    C: CodeHolder,
    C::Str: Into<Cow<'a, str>>,
{
    pub const fn new(inner: C) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, C> CodeHolder for InterningCodeHolder<'a, C>
where
    C: CodeHolder,
    C::Str: AsRef<str>,
{
    type Str = Interned<str>;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        let chunk = self.inner.get_chunk(at);

        TOTAL_SHARES.fetch_add(1, Ordering::AcqRel);
        GLOBAL_STRING_POOL.intern(chunk.as_ref())
    }
}
