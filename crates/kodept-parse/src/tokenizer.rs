use crate::common::ErrorAdapter;
use crate::error::{Original, ParseErrors};
use crate::lexer::DefaultLexer;
use crate::token_match::TokenMatch;
use std::fmt::Debug;

#[cfg(feature = "parallel")]
pub use parallel::Tokenizer as ParallelTokenizer;
pub use {eager::Tokenizer as EagerTokenizer, lazy::Tokenizer as LazyTokenizer};

pub trait Tokenizer<'t, F> {
    type Error;

    fn new(input: &'t str, lexer: F) -> Self;

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

pub trait TokenizerExt<'t> {
    fn default(input: &'t str) -> Self;
}

impl<'t, T: Tokenizer<'t, DefaultLexer>> TokenizerExt<'t> for T {
    fn default(input: &'t str) -> Self {
        T::new(input, DefaultLexer::new())
    }
}

mod lazy {
    use super::Tokenizer as Tok;
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

            let mut token_match = match self.tokenizing_fn.parse_token(self.buffer, self.pos) {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            };

            token_match.span.point.offset = self.pos;
            self.pos += token_match.span.point.length;

            Some(Ok(token_match))
        }
    }

    impl<'t, F> FusedIterator for Tokenizer<'t, F> where F: TokenProducer {}

    impl<'t, F> Tok<'t, F> for Tokenizer<'t, F>
    where
        F: TokenProducer,
    {
        type Error = F::Error<'t>;

        #[inline]
        fn new(input: &'t str, lexer: F) -> Self {
            Self {
                buffer: input,
                pos: 0,
                tokenizing_fn: lexer,
            }
        }

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
}

mod eager {
    use super::Tokenizer as Tok;
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

    impl<'t, F> Tok<'t, F> for Tokenizer<'t, F::Error<'t>, F>
    where
        F: EagerTokensProducer,
    {
        type Error = F::Error<'t>;

        fn new(input: &'t str, lexer: F) -> Self {
            let tokens = lexer.parse_tokens(input);
            Self {
                input,
                result: tokens,
                lexer_type: PhantomData,
            }
        }

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
}

#[cfg(feature = "parallel")]
mod parallel {
    use std::fmt::Debug;
    use std::iter::once;

    use itertools::Itertools;
    use rayon::prelude::*;

    use super::Tokenizer as Tok;
    use crate::common::{ErrorAdapter, TokenProducer};
    use crate::error::{Original, ParseErrors};
    use crate::token_match::TokenMatch;
    use crate::tokenizer::lazy;

    #[derive(Debug)]
    pub struct Tokenizer<'t, F> {
        input: &'t str,
        lines: Vec<(usize, &'t str)>,
        handler: F,
    }

    impl<'t, F> Tok<'t, F> for Tokenizer<'t, F>
    where
        F: TokenProducer + Clone + Sync,
        F::Error<'t>: Send,
    {
        type Error = F::Error<'t>;

        fn new(input: &'t str, lexer: F) -> Self {
            let mut lines = input.split_inclusive('\n').peekable();
            let Some(first) = lines.peek() else {
                return Self {
                    input,
                    lines: vec![],
                    handler: lexer,
                };
            };
            let lines: Vec<_> = once((0, *first))
                .chain(lines.tuple_windows().scan(0, |offset, (a, b)| {
                    *offset += a.len();
                    Some((*offset, b))
                }))
                .inspect(|it| println!("{}", it.0))
                .collect();

            Self {
                input,
                lines,
                handler: lexer,
            }
        }

        fn try_into_vec(self) -> Result<Vec<TokenMatch<'t>>, Self::Error> {
            self.lines
                .into_par_iter()
                .flat_map_iter(|(offset, line)| {
                    lazy::Tokenizer::new(line, self.handler.clone()).update(move |it| match it {
                        Ok(x) => x.span.point.offset += offset,
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
}
