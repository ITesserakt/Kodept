use crate::common::VerboseEnclosed;
use crate::lexer::traits::ToRepresentation;
use crate::lexer::PackedToken;
use crate::lexer::PackedToken::{
    Comma, LBrace, LParen, Newline, RBrace, RParen, Semicolon, Whitespace,
};
use crate::nom::parser::PParser;
use crate::nom::TokenVerificationError;
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use nom::branch::alt;
use nom::bytes::{take, take_while};
use nom::combinator::{cut, map, opt};
use nom::multi::{many1, separated_list0, separated_list1};
use nom::sequence::{preceded, terminated};
use nom::Parser;
use std::fmt::Debug;

#[inline(always)]
pub(super) fn match_token<'t>(example: PackedToken) -> impl PParser<'t, PackedTokenMatch> {
    preceded(
        take_while(|it: PackedTokenMatch| it.token.is_ignored()),
        take(1usize),
    )
    .map(|it: PackedTokenStream| it.into_single())
    .map_res(move |it| {
        (it.token == example)
            .then_some(it)
            .ok_or_else(|| TokenVerificationError::new(example.representation()))
    })
}

#[inline(always)]
pub(super) fn match_any_token<'t>(expected: PackedToken) -> impl PParser<'t, PackedTokenMatch> {
    take(1usize)
        .map(|it: PackedTokenStream| it.into_single())
.map_res(move |it| {
            (it.token == expected)
                .then_some(it)
                .ok_or(TokenVerificationError::new(expected.representation()))
        })
}

#[inline]
#[allow(unused)]
pub(super) fn inspect_input<I: Debug, P>(
    mut parser: P,
) -> impl Parser<I, Output = P::Output, Error = P::Error>
where
    P: Parser<I>,
{
    move |input| parser.parse(dbg!(input))
}

#[inline]
#[allow(unused)]
pub(super) fn inspect<I, P>(mut parser: P) -> impl Parser<I, Output = P::Output, Error = P::Error>
where
    P: Parser<I, Output: Debug>,
{
    move |input| {
        let (rest, result) = parser.parse(input)?;
        dbg!(&result);
        Ok((rest, result))
    }
}

#[inline]
pub(super) fn paren_enclosed<'t, T>(
    items_parser: impl PParser<'t, T>,
) -> impl PParser<'t, VerboseEnclosed<T>> {
    map(
        (match_token(LParen), cut(items_parser), match_token(RParen)),
        VerboseEnclosed::from,
    )
}

#[inline]
pub(super) fn brace_enclosed<'t, T>(
    items_parser: impl PParser<'t, T>,
) -> impl PParser<'t, VerboseEnclosed<T>> {
    map(
        (match_token(LBrace), cut(items_parser), match_token(RBrace)),
        VerboseEnclosed::from,
    )
}

#[allow(unused_parens)]
#[inline]
pub(super) fn newline_separated<'t, T>(
    items_parser: impl PParser<'t, T>,
) -> impl PParser<'t, Vec<T>> {
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
pub(super) fn comma_separated0<'t, T>(
    items_parser: impl PParser<'t, T>,
) -> impl PParser<'t, Vec<T>> {
    terminated(
        separated_list0(match_token(Comma), items_parser),
        opt(match_token(Comma)),
    )
}

#[inline]
pub(super) fn comma_separated1<'t, T>(
    items_parser: impl PParser<'t, T>,
) -> impl PParser<'t, Vec<T>> {
    terminated(
        separated_list1(match_token(Comma), items_parser),
        opt(match_token(Comma)),
    )
}
