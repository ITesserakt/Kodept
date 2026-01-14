use crate::error::{Original, ParseErrors};
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_rlt::new_types::Enclosed;
use kodept_rlt::prelude::RLT;
use std::fmt::Debug;
use std::marker::PhantomData;

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

    fn parse_stream<'t>(&self, input: &PackedTokenStream<'t>) -> Result<Output, Self::Error<'t>>;
}

pub struct Adapted<T, A>(pub T, PhantomData<A>);

impl<T, A> Adapted<T, A> {
    pub fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

impl<A, T> RLTProducer for Adapted<T, A>
where
    T: RLTProducer,
    for<'t> T::Error<'t>: ErrorAdapter<A, PackedTokenStream<'t>>,
    for<'t> PackedTokenStream<'t>: Original<A>,
{
    type Error<'t> = ParseErrors<A>;

    #[inline]
    fn parse_stream<'t>(&self, input: &PackedTokenStream<'t>) -> Result<RLT, Self::Error<'t>> {
        self.0.parse_stream(input).map_err(|e| e.adapt(*input, 0))
    }
}

impl<A, T> EagerTokensProducer for Adapted<T, A>
where
    T: EagerTokensProducer,
    for<'t> T::Error<'t>: ErrorAdapter<A, &'t str>,
    for<'t> &'t str: Original<A>,
{
    type Error<'t> = ParseErrors<A>;

    #[inline]
    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<PackedTokenMatch>, Self::Error<'t>> {
        self.0.parse_string(input).map_err(|e| e.adapt(input, 0))
    }
}

impl<A, T> TokenProducer for Adapted<T, A>
where
    T: TokenProducer,
    for<'t> T::Error<'t>: ErrorAdapter<A, &'t str>,
    for<'t> &'t str: Original<A>,
{
    type Error<'t> = ParseErrors<A>;

    fn parse_string<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<PackedTokenMatch, Self::Error<'t>> {
        self.0
            .parse_string(whole_input, position)
            .map_err(|e| e.adapt(whole_input, position))
    }
}

pub trait ErrorAdapter<A, O: Original<A>> {
    fn adapt(self, original_input: O, position: usize) -> ParseErrors<A>;
}

#[derive(Clone, Debug)]
pub struct VerboseEnclosed<T> {
    pub left: CodePoint,
    pub inner: T,
    pub right: CodePoint,
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
            left: left.location(),
            inner,
            right: right.location(),
        }
    }
}
