use nom::branch::alt;
use nom::Parser;
use nom_supreme::ParserExt;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::{function, match_token};
use crate::nom::parser::utils::{comma_separated0, paren_enclosed};
use crate::nom::parser::{operator, ParseResult};
use crate::token_stream::PackedTokenStream;
use kodept_core::structure::rlt;
use kodept_core::structure::span::Span;

fn tuple_literal(input: PackedTokenStream) -> ParseResult<rlt::Literal> {
    paren_enclosed(comma_separated0(operator::grammar))
        .context(function!())
        .map(|it| rlt::Literal::Tuple(it.into()))
        .parse(input)
}

pub(super) fn grammar(input: PackedTokenStream) -> ParseResult<rlt::Literal> {
    alt((
        match_token!(Binary).map(|it| rlt::Literal::Binary(Span::new(it.point))),
        match_token!(Octal).map(|it| rlt::Literal::Octal(Span::new(it.point))),
        match_token!(Hex).map(|it| rlt::Literal::Hex(Span::new(it.point))),
        match_token!(Floating).map(|it| rlt::Literal::Floating(Span::new(it.point))),
        match_token!(Char).map(|it| rlt::Literal::Char(Span::new(it.point))),
        match_token!(String).map(|it| rlt::Literal::String(Span::new(it.point))),
        tuple_literal,
    ))
    .context(function!())
    .parse(input)
}
