use crate::common::VerboseEnclosed;
use crate::lexer::traits::ToRepresentation;
use crate::lexer::PackedToken;
use crate::lexer::PackedToken::{
    Comma, LBrace, LParen, Newline, RBrace, RParen, Semicolon, Whitespace,
};
use crate::nom::parser::{ParseError, ParseResult};
use crate::nom::TokenVerificationError;
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use nom::branch::alt;
use nom::bytes::complete::{take, take_while};
use nom::multi::{many1, separated_list0, separated_list1};
use nom::sequence::tuple;
use nom::Err::Error;
use nom::IResult;
use nom::Parser;
use nom_supreme::error::BaseErrorKind;
use nom_supreme::ParserExt;
use std::fmt::Debug;

#[inline]
fn any_not_ignored_token(input: PackedTokenStream) -> ParseResult<PackedTokenStream> {
    take_while(|it: PackedTokenMatch| it.token.is_ignored())
        .precedes(take(1usize))
        .parse(input)
}

#[inline]
fn any_token(input: PackedTokenStream) -> ParseResult<PackedTokenStream> {
    take(1usize).parse(input)
}

#[inline(always)]
pub(super) fn match_token(
    example: PackedToken,
) -> impl FnMut(PackedTokenStream) -> ParseResult<PackedTokenMatch> {
    move |input| {
        let (rest, output) = any_not_ignored_token(input)?;
        let token_match = output.into_single();
        
        if token_match.token == example {
            Ok((rest, token_match))
        } else {
            let repr = example.representation();
            let error = ParseError::Base {
                location: output,
                kind: BaseErrorKind::External(TokenVerificationError::new(repr)),
            };
            Err(Error(error))
        }
    }
}

#[inline(always)]
pub(super) fn match_any_token(
    expected: PackedToken,
) -> impl FnMut(PackedTokenStream) -> ParseResult<PackedTokenMatch> {
    move |input| {
        let (rest, output) = any_token(input)?;
        let token_match = output.into_single();
        
        if token_match.token == expected {
            Ok((rest, token_match))
        } else {
            let repr = expected.representation();
            let error = ParseError::Base {
                location: output,
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
    separated_list0(
        many1(alt((
            match_any_token(Newline),
            match_any_token(Whitespace),
            match_token(Semicolon),
        ))),
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
