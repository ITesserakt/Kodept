use nom::multi::many0;
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::lexer::Keyword::{Elif, Else, If};
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::match_token;
use crate::nom::parser::{block_level, operator, ParseResult};
use crate::token_stream::TokenStream;

fn else_expr(input: TokenStream) -> ParseResult<rlt::ElseExpr> {
    tuple((match_token(Else), block_level::body.cut()))
        .context(function!())
        .map(|it| rlt::ElseExpr {
            keyword: it.0.span.into(),
            body: it.1,
        })
        .parse(input)
}

fn elif_expr(input: TokenStream) -> ParseResult<rlt::ElifExpr> {
    tuple((
        match_token(Elif),
        operator::grammar.cut(),
        block_level::body.cut(),
    ))
    .context(function!())
    .map(|it| rlt::ElifExpr {
        keyword: it.0.span.into(),
        condition: it.1,
        body: it.2,
    })
    .parse(input)
}

pub(super) fn if_expr(input: TokenStream) -> ParseResult<rlt::IfExpr> {
    tuple((
        match_token(If),
        operator::grammar.cut(),
        block_level::body.cut(),
        many0(elif_expr),
        else_expr.opt(),
    ))
    .context(function!())
    .map(|it| rlt::IfExpr {
        keyword: it.0.span.into(),
        condition: it.1,
        body: it.2,
        elif: it.3.into_boxed_slice(),
        el: it.4,
    })
    .parse(input)
}
