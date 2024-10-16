use std::fmt::Debug;

use kodept_core::structure::rlt::new_types::Enclosed;
use kodept_core::structure::rlt::RLT;
use kodept_core::structure::span::Span;
use kodept_core::structure::Located;

use crate::error::{Original, ParseErrors};
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;

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
    type Error<'t>
    where
        Self: 't;

    fn parse_stream<'t>(&self, input: &PackedTokenStream<'t>) -> Result<Output, Self::Error<'t>>;
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

impl<T> From<(PackedTokenMatch, T, PackedTokenMatch)> for VerboseEnclosed<T> {
    fn from((left, inner, right): (PackedTokenMatch, T, PackedTokenMatch)) -> Self {
        Self::from_located(left, inner, right)
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
