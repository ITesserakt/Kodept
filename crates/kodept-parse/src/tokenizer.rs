use crate::common::ErrorAdapter;
use crate::error::{Original, ParseErrors};
use crate::token_match::TokenMatch;
use std::fmt::Debug;

use crate::lexer::DefaultLexer;
#[cfg(feature = "parallel")]
pub use parallel::Tokenizer as ParallelTokenizer;
pub use {eager::Tokenizer as EagerTokenizer, lazy::Tokenizer as LazyTokenizer};

pub trait Tok<'t> {
    type Error;

    fn try_into_vec(self) -> Result<Vec<TokenMatch<'t>>, Self::Error>;

    fn try_collect_adapted<A>(self) -> Result<Vec<TokenMatch<'t>>, ParseErrors<A>>
    where
        &'t str: Original<A>,
        Self::Error: ErrorAdapter<A, &'t str>;

    fn into_vec(self) -> Vec<TokenMatch<'t>>
    where
        Self::Error: Debug,
        Self: Sized,
    {
        self.try_into_vec().unwrap()
    }
}

pub trait TokCtor<'t, F> {
    fn new(input: &'t str, lexer: F) -> Self;

    fn default(input: &'t str) -> Self
    where
        F: From<DefaultLexer>,
        Self: Sized,
    {
        Self::new(input, DefaultLexer::new().into())
    }
}

mod lazy {
    use super::{Tok, TokCtor};
    use crate::common::{ErrorAdapter, TokenProducer};
    use crate::error::{Original, ParseErrors};
    use crate::token_match::TokenMatch;
    use std::iter::FusedIterator;

    pub struct Tokenizer<'t, F> {
        buffer: &'t str,
        pos: usize,
        tokenizing_fn: F,
    }

    impl<'t, F> Iterator for Tokenizer<'t, F>
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

            let mut token_match = match self.tokenizing_fn.parse_string(self.buffer, self.pos) {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            };

            token_match.span.point.offset = self.pos as u32;
            self.pos += token_match.span.point.length as usize;

            Some(Ok(token_match))
        }
    }

    impl<'t, F> FusedIterator for Tokenizer<'t, F> where F: TokenProducer {}

    impl<'t, F> Tok<'t> for Tokenizer<'t, F>
    where
        F: TokenProducer,
    {
        type Error = F::Error<'t>;

        fn try_into_vec(self) -> Result<Vec<TokenMatch<'t>>, Self::Error> {
            let vec: Result<Vec<_>, _> = <Self as Iterator>::collect(self);
            let mut vec = vec?;
            vec.shrink_to_fit();
            Ok(vec)
        }

        fn try_collect_adapted<A>(self) -> Result<Vec<TokenMatch<'t>>, ParseErrors<A>>
        where
            &'t str: Original<A>,
            Self::Error: ErrorAdapter<A, &'t str>,
        {
            let input = self.buffer;
            let pos = self.pos;
            self.try_into_vec().map_err(|e| e.adapt(input, pos))
        }
    }

    impl<'t, F> TokCtor<'t, F> for Tokenizer<'t, F> {
        fn new(input: &'t str, lexer: F) -> Self {
            Self {
                buffer: input,
                pos: 0,
                tokenizing_fn: lexer,
            }
        }
    }
}

mod eager {
    use super::{Tok, TokCtor};
    use crate::common::{EagerTokensProducer, ErrorAdapter};
    use crate::error::{Original, ParseErrors};
    use crate::token_match::TokenMatch;
    use std::fmt::Debug;
    use std::marker::PhantomData;

    #[derive(Debug)]
    pub struct Tokenizer<'t, E, F> {
        input: &'t str,
        result: Result<Vec<TokenMatch<'t>>, E>,
        lexer_type: PhantomData<F>,
    }

    impl<'t, F> Tok<'t> for Tokenizer<'t, F::Error<'t>, F>
    where
        F: EagerTokensProducer,
    {
        type Error = F::Error<'t>;

        #[inline]
        fn try_into_vec(self) -> Result<Vec<TokenMatch<'t>>, Self::Error> {
            self.result
        }

        fn try_collect_adapted<A>(self) -> Result<Vec<TokenMatch<'t>>, ParseErrors<A>>
        where
            &'t str: Original<A>,
            Self::Error: ErrorAdapter<A, &'t str>,
        {
            self.result.map_err(|e| e.adapt(self.input, 0))
        }
    }

    impl<'t, F> TokCtor<'t, F> for Tokenizer<'t, F::Error<'t>, F>
    where
        F: EagerTokensProducer,
    {
        fn new(input: &'t str, lexer: F) -> Self {
            let tokens = lexer.parse_string(input);
            Self {
                input,
                result: tokens,
                lexer_type: PhantomData,
            }
        }
    }
}

#[cfg(feature = "parallel")]
mod parallel {
    use super::{Tok, TokCtor};
    use crate::common::{ErrorAdapter, TokenProducer};
    use crate::error::{Original, ParseErrors};
    use crate::token_match::TokenMatch;
    use crate::tokenizer::lazy;
    use itertools::Itertools;
    use rayon::prelude::*;
    use std::fmt::Debug;

    const CHUNK_SIZE: usize = 120;

    #[derive(Debug)]
    pub struct Tokenizer<'t, F> {
        input: &'t str,
        lines: Vec<(usize, &'t str)>,
        handler: F,
    }

    impl<'t, F> Tok<'t> for Tokenizer<'t, F>
    where
        F: TokenProducer + Copy + Sync,
        F::Error<'t>: Send,
    {
        type Error = F::Error<'t>;

        fn try_into_vec(self) -> Result<Vec<TokenMatch<'t>>, Self::Error> {
            self.lines
                .into_par_iter()
                .flat_map_iter(|(offset, line)| {
                    lazy::Tokenizer::new(line, self.handler).update(move |it| match it {
                        Ok(x) => x.span.point.offset += offset as u32,
                        _ => {}
                    })
                })
                .collect()
        }

        fn try_collect_adapted<A>(self) -> Result<Vec<TokenMatch<'t>>, ParseErrors<A>>
        where
            &'t str: Original<A>,
            Self::Error: ErrorAdapter<A, &'t str>,
        {
            let input = self.input;
            self.try_into_vec().map_err(|e| e.adapt(input, 0))
        }
    }

    impl<'t, F> TokCtor<'t, F> for Tokenizer<'t, F> {
        fn new(input: &'t str, lexer: F) -> Self {
            let mut lines = vec![];
            let mut offset = 0;
            for (idx, ch) in input.char_indices() {
                let len = idx - offset;
                if len > CHUNK_SIZE && matches!(ch, '\n' | '\t' | ';' | ' ') {
                    lines.push((offset, &input[offset..idx]));
                    offset = idx;
                }
            }
            lines.push((offset, &input[offset..]));

            Self {
                input,
                lines,
                handler: lexer,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::lexer::DefaultLexer;
        use crate::tokenizer::TokCtor;

        #[test]
        fn test_split() {
            let input = "123\n1234\n\n1";
            let tokenizer = super::Tokenizer::new(input, DefaultLexer::new());

            assert_eq!(tokenizer.lines, vec![(0, "123\n1234\n\n1")]);
        }
    }
}
