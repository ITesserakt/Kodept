use nom::combinator::{cut, opt};
use nom::error::context;
use nom::multi::many0;
use nom::Parser;

use kodept_rlt::new_types::Keyword;
use kodept_rlt::prelude as rlt;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::match_token;
use crate::nom::parser::{block_level, operator, PParser};

fn else_expr<'t>() -> impl PParser<'t, rlt::ElseExpr> {
    context(function!(), (match_token(Else), cut(block_level::body()))).map(|it| rlt::ElseExpr {
        keyword: Keyword::from_located(it.0),
        body: it.1,
    })
}

fn elif_expr<'t>() -> impl PParser<'t, rlt::ElifExpr> {
    context(
        function!(),
        (
            match_token(Elif),
            cut(operator::grammar()),
            cut(block_level::body()),
        ),
    )
    .map(|it| rlt::ElifExpr {
        keyword: Keyword::from_located(it.0),
        condition: it.1,
        body: it.2,
    })
}

pub(super) fn if_expr<'t>() -> impl PParser<'t, rlt::IfExpr> {
    context(
        function!(),
        (
            match_token(If),
            cut(operator::grammar()),
            cut(block_level::body()),
            many0(elif_expr()),
            opt(else_expr()),
        ),
    )
    .map(|it| rlt::IfExpr {
        keyword: Keyword::from_located(it.0),
        condition: it.1,
        body: it.2,
        elif: it.3.into_boxed_slice(),
        el: it.4,
    })
}
