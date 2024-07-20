use std::borrow::Cow;
use std::fmt::Debug;
use std::iter::FusedIterator;

use tracing::error;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;

use crate::error::ParseErrors;
use crate::lexer::Token;
use crate::token_match::TokenMatch;

#[cfg(all(not(feature = "peg"), not(feature = "pest"), feature = "nom"))]
pub type Tokenizer<'t> = simple_implementation::Tokenizer<'t>;

#[cfg(any(feature = "peg", feature = "pest"))]
pub type Tokenizer<'t> = crate::grammar::KodeptParser<'t>;

#[cfg(all(feature = "peg", feature = "trace"))]
pub type TracedTokenizer<'t> = crate::grammar::peg::Tokenizer<'t, true>;

pub struct LazyTokenizer;

impl LazyTokenizer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'t>(
        reader: &'t str,
    ) -> GenericLazyTokenizer<
        impl FnMut(&'t str, &'t str) -> TokenizingResult<'t, ParseErrors<Cow<'t, str>>>,
    > {
        GenericLazyTokenizer::new(reader, crate::lexer::parse_token)
    }
}

type TokenizingResult<'t, E> = Result<TokenMatch<'t>, E>;

pub struct GenericLazyTokenizer<'t, F> {
    buffer: &'t str,
    pos: usize,
    tokenizing_fn: F,
}

impl<'t, F> GenericLazyTokenizer<'t, F> {
    #[inline]
    #[must_use]
    pub const fn new(reader: &'t str, tokenizing_fn: F) -> Self {
        Self {
            buffer: reader,
            pos: 0,
            tokenizing_fn,
        }
    }

    pub fn into_vec<E>(self) -> Vec<TokenMatch<'t>>
    where
        F: FnMut(&'t str, &'t str) -> TokenizingResult<'t, E>,
        E: Debug,
    {
        let mut vec: Vec<_> = self.collect();
        vec.shrink_to_fit();
        vec
    }
}

impl<'t, E: Debug, F: FnMut(&'t str, &'t str) -> TokenizingResult<'t, E>> Iterator
    for GenericLazyTokenizer<'t, F>
{
    type Item = TokenMatch<'t>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer[self.pos..].is_empty() {
            return None;
        }

        let mut token_match = (self.tokenizing_fn)(&self.buffer[self.pos..], self.buffer)
            .unwrap_or_else(|e| {
                error!("Cannot parse token: {e:#?}");
                TokenMatch::new(Token::Unknown, Span::new(CodePoint::single_point(self.pos)))
            });

        token_match.span.point.offset = self.pos;
        self.pos += token_match.span.point.length;

        Some(token_match)
    }
}

impl<'t, E: Debug, F: FnMut(&'t str, &'t str) -> TokenizingResult<'t, E>> FusedIterator
    for GenericLazyTokenizer<'t, F>
{
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use kodept_core::code_point::CodePoint;

    use crate::lexer::{
        Identifier::*, Ignore::*, Keyword::*, MathOperator::*, Operator::*, Symbol::*,
    };
    use crate::tokenizer::Tokenizer;

    #[test]
    fn test_tokenizer_simple() {
        let input = " fun foo(x: Int, y: Int) => \n  x + y";
        let tokenizer = Tokenizer::new(input);
        let spans: Vec<_> = tokenizer.collect();

        assert_eq!(spans.len(), 26);
        assert_eq!(
            spans.iter().map(|it| it.token).collect::<Vec<_>>(),
            vec![
                Whitespace.into(),
                Fun.into(),
                Whitespace.into(),
                Identifier("foo").into(),
                LParen.into(),
                Identifier("x").into(),
                Colon.into(),
                Whitespace.into(),
                Type("Int").into(),
                Comma.into(),
                Whitespace.into(),
                Identifier("y").into(),
                Colon.into(),
                Whitespace.into(),
                Type("Int").into(),
                RParen.into(),
                Whitespace.into(),
                Flow.into(),
                Whitespace.into(),
                Newline.into(),
                Whitespace.into(),
                Identifier("x").into(),
                Whitespace.into(),
                Math(Plus).into(),
                Whitespace.into(),
                Identifier("y").into()
            ]
        );
        assert_eq!(spans.get(20).unwrap().span.point, CodePoint::new(2, 29))
    }
}
