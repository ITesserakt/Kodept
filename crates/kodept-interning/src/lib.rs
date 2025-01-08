pub mod metrics;

use interner::global::{GlobalString, StringPool};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_core::shared_str::{Stringy, SharedStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

static GLOBAL_STRING_POOL: StringPool = StringPool::new();
static TOTAL_SHARES: AtomicUsize = AtomicUsize::new(0);

#[repr(transparent)]
struct Helper(GlobalString);

#[allow(unsafe_code)]
unsafe impl Stringy for Helper {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }

    fn clone(&self) -> SharedStr {
        TOTAL_SHARES.fetch_add(1, Ordering::Relaxed);
        SharedStr::new(Helper(self.0.clone()))
    }
}

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
    C::Str: Into<Cow<'a, str>>,
{
    type Str = SharedStr;

    fn get_chunk(self, at: CodePoint) -> SharedStr {
        let chunk = self.inner.get_chunk(at);

        TOTAL_SHARES.fetch_add(1, Ordering::AcqRel);
        SharedStr::new(Helper(GLOBAL_STRING_POOL.get(chunk)))
    }
}

impl Drop for Helper {
    fn drop(&mut self) {
        TOTAL_SHARES.fetch_sub(1, Ordering::Relaxed);
    }
}
