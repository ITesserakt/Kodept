use std::fmt::Debug;

use itertools::Itertools;

use crate::common::{ErrorAdapter, TokenProducer};
use crate::error::{Original, ParseErrors};
use crate::lexer::DefaultLexer;
use crate::token_match::TokenMatch;

pub struct LazyTokenizer;

impl LazyTokenizer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(input: &str) -> GenericLazyTokenizer<DefaultLexer> {
        GenericLazyTokenizer::new(input, DefaultLexer::new())
    }
}

pub struct GenericLazyTokenizer<'t, F> {
    buffer: &'t str,
    pos: usize,
    tokenizing_fn: F,
}

impl<'t, F> GenericLazyTokenizer<'t, F> {
    pub const fn new(reader: &'t str, tokenizing_fn: F) -> Self {
        Self {
            buffer: reader,
            pos: 0,
            tokenizing_fn,
        }
    }

    pub fn try_into_vec<A>(self) -> Result<Vec<TokenMatch<'t>>, ParseErrors<A>>
    where
        F: TokenProducer<Error<'t>: ErrorAdapter<A, &'t str>>,
        &'t str: Original<A>
    {
        let buf = self.buffer;
        let pos = self.pos;
        match self.try_collect::<_, Vec<_>, _>() {
            Ok(x) => Ok(x),
            Err(e) => Err(e.adapt(buf, pos))
        }
    }

    pub fn into_vec(self) -> Vec<TokenMatch<'t>>
    where
        F: TokenProducer<Error<'t>: Debug>,
    {
        self.try_collect().unwrap()
    }
}

impl<'t, F> Iterator for GenericLazyTokenizer<'t, F>
where
    F: TokenProducer,
{
    type Item = Result<TokenMatch<'t>, F::Error<'t>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let slice = &self.buffer[self.pos..];
        if slice.is_empty() {
            return None;
        }

        let mut token_match = match self.tokenizing_fn.parse_token(self.buffer, self.pos) {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        };

        token_match.span.point.offset = self.pos;
        self.pos += token_match.span.point.length;

        Some(Ok(token_match))
    }
}
