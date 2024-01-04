use nom::branch::alt;
use nom::multi::many0;
use nom::Parser;
use nom::sequence::tuple;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::{function, ParseResult};
use crate::lexer::Keyword::{Elif, Else, If};
use crate::parser::{block_level, operator};
use crate::parser::nom::match_token;
use crate::token_stream::TokenStream;

fn else_expr(input: TokenStream) -> ParseResult<rlt::ElseExpr> {
    tuple((match_token(Else), block_level::body))
        .context(function!())
        .map(|it| rlt::ElseExpr {
            keyword: it.0.span.into(),
            body: it.1,
        })
        .parse(input)
}

fn elif_expr(input: TokenStream) -> ParseResult<rlt::ElifExpr> {
    tuple((match_token(Elif), operator::grammar, block_level::body))
        .context(function!())
        .map(|it| rlt::ElifExpr {
            keyword: it.0.span.into(),
            condition: it.1,
            body: it.2,
        })
        .parse(input)
}

pub fn if_expr(input: TokenStream) -> ParseResult<rlt::IfExpr> {
    tuple((
        match_token(If),
        operator::grammar,
        block_level::body,
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

#[allow(unused)]
// TODO
pub fn grammar(input: TokenStream) -> ParseResult<rlt::CodeFlow> {
    alt((if_expr.map(rlt::CodeFlow::If),))
        .context(function!())
        .parse(input)
}
