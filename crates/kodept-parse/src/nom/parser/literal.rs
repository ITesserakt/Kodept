use nom::branch::alt;
use nom::error::context;
use nom::Parser;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::{operator, PParser};
use kodept_rlt::prelude as rlt;

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
            match_token(Binary).map(|it| rlt::Literal::Binary(it.point)),
            match_token(Octal).map(|it| rlt::Literal::Octal(it.point)),
            match_token(Hex).map(|it| rlt::Literal::Hex(it.point)),
            match_token(Floating).map(|it| rlt::Literal::Floating(it.point)),
            match_token(Char).map(|it| rlt::Literal::Char(it.point)),
            match_token(String).map(|it| rlt::Literal::String(it.point)),
            tuple_literal(),
        )),
    )
}
