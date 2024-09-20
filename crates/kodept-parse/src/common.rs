use std::fmt::Debug;

use kodept_core::structure::rlt::new_types::Enclosed;
use kodept_core::structure::rlt::RLT;
use kodept_core::structure::span::Span;
use kodept_core::structure::Located;

use crate::error::{Original, ParseErrors};
use crate::token_match::{PackedTokenMatch, TokenMatch};
use crate::token_stream::TokenStream;

pub trait TokenProducer {
    type Error<'t>;

    fn parse_string<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<PackedTokenMatch, Self::Error<'t>>;
}

pub trait EagerTokensProducer {
    type Error<'t>;
    
    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<PackedTokenMatch>, Self::Error<'t>>;
}

pub trait RLTProducer<Output = RLT> {
    type Error<'t>;

    fn parse_stream<'t>(&self, input: TokenStream<'t>) -> Result<Output, Self::Error<'t>>;
}

pub trait ErrorAdapter<A, O: Original<A>> {
    fn adapt(self, original_input: O, position: usize) -> ParseErrors<A>;
}

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
    fn from(value: (TokenMatch<'t>, T, TokenMatch<'t>)) -> Self {
        Self {
            left: value.0.span,
            inner: value.1,
            right: value.2.span,
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
