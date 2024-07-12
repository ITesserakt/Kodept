use std::fmt::Debug;

use derive_more::{Constructor};
use nom::bytes::complete::{take, take_while};
use nom::Err::Error;
use nom::IResult;
use nom::multi::{separated_list0, separated_list1};
use nom::Parser;
use nom::sequence::tuple;
use nom_supreme::error::BaseErrorKind;
use nom_supreme::ParserExt;
use thiserror::Error;

use crate::{
    lexer::Token, ParseError, ParseResult, token_match::TokenMatch, token_stream::TokenStream,
};
use crate::lexer::traits::ToRepresentation;
use crate::parser::common::VerboseEnclosed;

#[inline]
pub fn any_not_ignored_token(input: TokenStream) -> ParseResult<TokenMatch> {
    take_while(|it: TokenMatch| matches!(it.token, Token::Ignore(_)))
        .precedes(take(1usize))
        .map(|it: TokenStream| {
            it.into_token_match()
                .expect("Token stream with 1 element can be coerced to lexer match")
        })
        .parse(input)
}

#[inline]
pub fn any_token(input: TokenStream) -> ParseResult<TokenMatch> {
    take(1usize)
        .map(|it: TokenStream| {
            it.into_token_match()
                .expect("Token stream with 1 element can be coerced to lexer match")
        })
        .parse(input)
}

#[derive(Error, Debug, Constructor)]
#[error("Expected `{expected}`")]
pub struct TokenVerificationError {
    pub expected: &'static str,
}

#[inline]
pub fn match_token<'t, T>(example: T) -> impl FnMut(TokenStream<'t>) -> ParseResult<TokenMatch<'t>>
where
    T: Into<Token<'t>> + Clone + ToRepresentation,
{
    let repr = example.representation();
    move |input| {
        let example = example.clone();
        let i = input.clone();
        let (input, output) = any_not_ignored_token(input)?;

        if output.token == example.into() {
            Ok((input, output))
        } else {
            let error = ParseError::Base {
                location: i,
                kind: BaseErrorKind::External(TokenVerificationError::new(repr)),
            };
            Err(Error(error))
        }
    }
}

#[macro_export]
macro_rules! match_token {
    ($pat:pat_param) => {{
        nom::error::context(
            stringify!($pat),
            nom::combinator::verify($crate::parser::nom::any_not_ignored_token, move |t| {
                matches!(&t.token, $pat)
            }),
        )
    }};
}

#[macro_export]
macro_rules! match_any_token {
    ($pat:pat_param) => {{
        nom::error::context(
            stringify!($pat),
            nom::combinator::verify($crate::parser::nom::any_token, move |t| {
                matches!(&t.token, $pat)
            }),
        )
    }};
}

#[macro_export]
#[cfg(test)]
macro_rules! assert_parses_to {
    ($parser:ident, $input:expr, $expectation:pat_param) => {
        match $parser($input) {
            Err(::nom::Err::Error(e) | ::nom::Err::Failure(e)) => {
                panic!("{}", ::nom::error::convert_error($input, e));
            }
            Err(e) => {
                panic!("Failed to parse {:?}", e)
            }
            Ok((_, candidate_val)) => {
                if !matches!(&candidate_val, $expectation) {
                    panic!(
                        "Failed to parse to expected value\n\
                        Got:      {:?}",
                        &candidate_val
                    )
                }
            }
        }
    };
}

#[inline]
#[allow(unused)]
pub fn inspect_input<I: Debug, O, E, P>(mut parser: P) -> impl FnMut(I) -> IResult<I, O, E>
where
    P: Parser<I, O, E>,
{
    move |input| parser.parse(dbg!(input))
}

#[inline]
#[allow(unused)]
pub fn inspect<I: Debug, O: Debug, E: Debug, P>(mut parser: P) -> impl FnMut(I) -> IResult<I, O, E>
where
    P: Parser<I, O, E>,
{
    move |input| {
        let (rest, result) = parser.parse(input)?;
        dbg!(&result);
        Ok((rest, result))
    }
}

#[inline]
pub fn paren_enclosed<'t, T, P: Parser<TokenStream<'t>, T, ParseError<'t>>>(
    items_parser: P,
) -> impl Parser<TokenStream<'t>, VerboseEnclosed<'t, T>, ParseError<'t>> {
    use crate::lexer::Symbol::*;

    tuple((
        match_token(LParen),
        items_parser.cut(),
        match_token(RParen).cut(),
    ))
    .map(|it| it.into())
}

#[inline]
pub fn brace_enclosed<'t, T, P: Parser<TokenStream<'t>, T, ParseError<'t>>>(
    items_parser: P,
) -> impl Parser<TokenStream<'t>, VerboseEnclosed<'t, T>, ParseError<'t>> {
    use crate::lexer::Symbol::*;

    tuple((
        match_token(LBrace),
        items_parser.cut(),
        match_token(RBrace).cut(),
    ))
    .map(|it| it.into())
}

#[allow(unused_parens)]
#[inline]
pub fn newline_separated<'t, T, P: Parser<TokenStream<'t>, T, ParseError<'t>>>(
    items_parser: P,
) -> impl Parser<TokenStream<'t>, Vec<T>, ParseError<'t>> {
    use crate::lexer::{Ignore::*, Symbol::*};

    separated_list0(
        match_any_token!((Token::Ignore(Newline | Whitespace) | Token::Symbol(Semicolon))),
        items_parser,
    )
}

#[inline]
pub fn comma_separated0<'t, T, P: Parser<TokenStream<'t>, T, ParseError<'t>>>(
    items_parser: P,
) -> impl Parser<TokenStream<'t>, Vec<T>, ParseError<'t>> {
    use crate::lexer::Symbol::*;

    separated_list0(match_token(Comma), items_parser).terminated(match_token(Comma).opt())
}

#[inline]
pub fn comma_separated1<'t, T, P: Parser<TokenStream<'t>, T, ParseError<'t>>>(
    items_parser: P,
) -> impl Parser<TokenStream<'t>, Vec<T>, ParseError<'t>> {
    use crate::lexer::Symbol::*;

    separated_list1(match_token(Comma), items_parser).terminated(match_token(Comma).opt())
}
