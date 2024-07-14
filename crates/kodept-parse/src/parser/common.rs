use kodept_core::structure::Located;
use kodept_core::structure::rlt::new_types::Enclosed;
use kodept_core::structure::span::Span;
use crate::token_match::TokenMatch;

#[derive(Clone, Debug)]
pub struct VerboseEnclosed<T> {
    pub left: Span,
    pub inner: T,
    pub right: Span,
}

impl<T, U: From<T>> From<VerboseEnclosed<T>> for Enclosed<U> {
    #[inline]
    fn from(value: VerboseEnclosed<T>) -> Self {
        Self {
            left: value.left.into(),
            inner: value.inner.into(),
            right: value.right.into(),
        }
    }
}

impl<'t, T> From<(TokenMatch<'t>, T, TokenMatch<'t>)> for VerboseEnclosed<T> {
    #[inline]
    fn from(original: (TokenMatch<'t>, T, TokenMatch<'t>)) -> VerboseEnclosed<T> {
        VerboseEnclosed {
            left: original.0.span,
            inner: original.1,
            right: original.2.span,
        }
    }
}

impl<T> VerboseEnclosed<T> {
    pub fn from_located<L: Located>(left: L, inner: T, right: L) -> Self {
        Self {
            left: Span::new(left.location()),
            inner,
            right: Span::new(right.location()),
        }
    }
}
