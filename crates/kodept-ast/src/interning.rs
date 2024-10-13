use derive_more::{Deref, Display};
use interner::global::{GlobalString, StringPool};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;
use kodept_core::static_assert_size;

#[derive(Debug, Display, PartialEq, Ord, PartialOrd, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "String"))]
#[cfg_attr(feature = "serde", serde(from = "Cow<str>"))]
pub struct SharedStr(GlobalString);

static_assert_size!(SharedStr, 8);

static GLOBAL_STRING_POOL: StringPool = StringPool::new();
static TOTAL_SHARES: AtomicUsize = AtomicUsize::new(0);

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

#[derive(Copy, Clone)]
pub struct InterningCodeHolder<'a, C> {
    inner: C,
    _phantom: PhantomData<&'a ()>,
}

const fn humanize(mut value: f64) -> (f64, &'static str) {
    const SUFFIXES: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    
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

pub fn debug_interning_efficiency() {
    let total_shares = TOTAL_SHARES.load(Ordering::Acquire);
    let pooled_entries: CollectProperties = GLOBAL_STRING_POOL.pooled();
    let coefficient = total_shares as f64 / (pooled_entries.count as f64);

    let total_allocated_size_for_indexes = total_shares * size_of::<SharedStr>();
    let total_allocated_size_for_strings =
        pooled_entries.count * size_of::<Box<str>>() + pooled_entries.total_size;
    let (memory_saved, suffix) = humanize(
        total_allocated_size_for_strings as f64 * coefficient
            - total_allocated_size_for_indexes as f64,
    );

    debug!(
        "Approximate amount of memory saved by string interning: {:.1}{}",
        memory_saved, suffix
    );
}

impl<'a, C> InterningCodeHolder<'a, C>
where
    C: CodeHolder,
    C::Str: Into<Cow<'a, str>>,
{
    pub fn new(inner: C) -> Self {
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
        SharedStr(GLOBAL_STRING_POOL.get(chunk))
    }
}

impl From<Cow<'_, str>> for SharedStr {
    fn from(value: Cow<'_, str>) -> Self {
        TOTAL_SHARES.fetch_add(1, Ordering::AcqRel);
        Self(GLOBAL_STRING_POOL.get(value))
    }
}

impl SharedStr {
    pub fn new<'a>(value: impl Into<Cow<'a, str>>) -> Self {
        TOTAL_SHARES.fetch_add(1, Ordering::AcqRel);
        Self(GLOBAL_STRING_POOL.get(value))
    }
}

impl AsRef<str> for SharedStr {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<SharedStr> for String {
    fn from(value: SharedStr) -> Self {
        value.to_string()
    }
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<&SharedStr> for String {
    fn from(value: &SharedStr) -> Self {
        value.to_string()
    }
}

impl Drop for SharedStr {
    fn drop(&mut self) {
        TOTAL_SHARES.fetch_sub(1, Ordering::AcqRel);
    }
}
