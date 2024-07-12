use derive_more::From;
use kodept_core::structure::rlt::new_types::Enclosed;
use crate::token_match::TokenMatch;

#[derive(From, Clone, Debug)]
pub struct VerboseEnclosed<'t, T> {
    pub left: TokenMatch<'t>,
    pub inner: T,
    pub right: TokenMatch<'t>,
}

impl<'t, T, U: From<T>> From<VerboseEnclosed<'t, T>> for Enclosed<U> {
    #[inline]
    fn from(value: VerboseEnclosed<'t, T>) -> Self {
        Self {
            left: value.left.span.into(),
            inner: value.inner.into(),
            right: value.right.span.into(),
        }
    }
}