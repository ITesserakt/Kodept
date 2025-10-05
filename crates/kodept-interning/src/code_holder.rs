use crate::{GlobalInterner, Interned};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::CodeHolder;

#[derive(Copy, Clone)]
pub struct InterningCodeHolder<C> {
    inner: C,
}

impl<C> InterningCodeHolder<C>
where
    C: CodeHolder,
    C::Str: AsRef<str>,
{
    pub const fn new(inner: C) -> Self {
        Self { inner }
    }
}

impl<C> CodeHolder for InterningCodeHolder<C>
where
    C: CodeHolder,
    C::Str: AsRef<str>,
{
    type Str = Interned<str>;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        let chunk = self.inner.get_chunk(at);

        str::interner().intern(chunk.as_ref())
    }
}
