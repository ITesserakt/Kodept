use std::fmt::Debug;

use crate::common::VerboseEnclosed;
use crate::lexer::traits::ToRepresentation;
use crate::lexer::PackedToken;
use crate::lexer::PackedToken::{Comma, LBrace, LParen, RBrace, RParen};
use crate::nom::parser::macros::match_any_token;
use crate::nom::parser::{ParseError, ParseResult};
use crate::nom::TokenVerificationError;
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use nom::bytes::complete::{take, take_while};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::tuple;
use nom::Err::Error;
use nom::IResult;
use nom::Parser;
use nom_supreme::error::BaseErrorKind;
use nom_supreme::ParserExt;

#[inline]
pub(super) fn any_not_ignored_token(input: PackedTokenStream) -> ParseResult<PackedTokenMatch> {
    take_while(|it: PackedTokenMatch| it.token.is_ignored())
        .precedes(take(1usize))
        .map(|it: PackedTokenStream| it.into_single())
        .parse(input)
}

#[inline]
pub(super) fn any_token(input: PackedTokenStream) -> ParseResult<PackedTokenMatch> {
    take(1usize)
        .map(|it: PackedTokenStream| it.into_single())
        .parse(input)
}

#[inline]
pub(super) fn match_token(
    example: PackedToken,
) -> impl FnMut(PackedTokenStream) -> ParseResult<PackedTokenMatch> {
    let repr = example.representation();
    move |input| {
        let i = input;
        let (input, output) = any_not_ignored_token(input)?;

        if output.token == example {
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

#[inline]
#[allow(unused)]
pub(super) fn inspect_input<I: Debug, O, E, P>(mut parser: P) -> impl FnMut(I) -> IResult<I, O, E>
where
    P: Parser<I, O, E>,
{
    move |input| parser.parse(dbg!(input))
}

#[inline]
#[allow(unused)]
pub(super) fn inspect<I: Debug, O: Debug, E: Debug, P>(
    mut parser: P,
) -> impl FnMut(I) -> IResult<I, O, E>
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
pub(super) fn paren_enclosed<'t, T, P>(
    items_parser: P,
) -> impl Parser<PackedTokenStream<'t>, VerboseEnclosed<T>, ParseError<'t>>
where
    P: Parser<PackedTokenStream<'t>, T, ParseError<'t>>,
{
    tuple((
        match_token(LParen),
        items_parser.cut(),
        match_token(RParen).cut(),
    ))
    .map(|it| it.into())
}

#[inline]
pub(super) fn brace_enclosed<'t, T, P>(
    items_parser: P,
) -> impl Parser<PackedTokenStream<'t>, VerboseEnclosed<T>, ParseError<'t>>
where
    P: Parser<PackedTokenStream<'t>, T, ParseError<'t>>,
{
    tuple((
        match_token(LBrace),
        items_parser.cut(),
        match_token(RBrace).cut(),
    ))
    .map(|it| it.into())
}

#[allow(unused_parens)]
#[inline]
pub(super) fn newline_separated<'t, T, P: Parser<PackedTokenStream<'t>, T, ParseError<'t>>>(
    items_parser: P,
) -> impl Parser<PackedTokenStream<'t>, Vec<T>, ParseError<'t>> {
    use crate::lexer::{Ignore::*, Symbol::*};

    separated_list0(
        match_any_token!((PackedToken::Newline | PackedToken::Whitespace | PackedToken::Semicolon)),
        items_parser,
    )
}

#[inline]
pub(super) fn comma_separated0<'t, T, P>(
    items_parser: P,
) -> impl Parser<PackedTokenStream<'t>, Vec<T>, ParseError<'t>>
where
    P: Parser<PackedTokenStream<'t>, T, ParseError<'t>>,
{
    separated_list0(match_token(Comma), items_parser).terminated(match_token(Comma).opt())
}

#[inline]
pub(super) fn comma_separated1<'t, T, P>(
    items_parser: P,
) -> impl Parser<PackedTokenStream<'t>, Vec<T>, ParseError<'t>>
where
    P: Parser<PackedTokenStream<'t>, T, ParseError<'t>>,
{
    separated_list1(match_token(Comma), items_parser).terminated(match_token(Comma).opt())
}
