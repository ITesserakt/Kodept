use nom::branch::alt;
use nom::error::context;
use nom::Parser;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::{operator, PParser};
use kodept_core::structure::rlt;
use kodept_core::structure::span::Span;

fn tuple_literal<'t>() -> impl PParser<'t, rlt::Literal> {
    context(
        function!(),
        paren_enclosed(comma_separated0(operator::grammar())),
    )
    .map(|it| rlt::Literal::Tuple(it.into()))
}

pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::Literal> {
    context(
        function!(),
        alt((
            match_token(Binary).map(|it| rlt::Literal::Binary(Span::new(it.point))),
            match_token(Octal).map(|it| rlt::Literal::Octal(Span::new(it.point))),
            match_token(Hex).map(|it| rlt::Literal::Hex(Span::new(it.point))),
            match_token(Floating).map(|it| rlt::Literal::Floating(Span::new(it.point))),
            match_token(Char).map(|it| rlt::Literal::Char(Span::new(it.point))),
            match_token(String).map(|it| rlt::Literal::String(Span::new(it.point))),
            tuple_literal(),
        )),
    )
}
