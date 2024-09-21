use nom::multi::many0;
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::Keyword;
use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::match_token;
use crate::nom::parser::{block_level, operator, ParseResult};
use crate::token_stream::PackedTokenStream;

fn else_expr(input: PackedTokenStream) -> ParseResult<rlt::ElseExpr> {
    tuple((match_token(Else), block_level::body.cut()))
        .context(function!())
        .map(|it| rlt::ElseExpr {
            keyword: Keyword::from_located(it.0),
            body: it.1,
        })
        .parse(input)
}

fn elif_expr(input: PackedTokenStream) -> ParseResult<rlt::ElifExpr> {
    tuple((
        match_token(Elif),
        operator::grammar.cut(),
        block_level::body.cut(),
    ))
    .context(function!())
    .map(|it| rlt::ElifExpr {
        keyword: Keyword::from_located(it.0),
        condition: it.1,
        body: it.2,
    })
    .parse(input)
}

pub(super) fn if_expr(input: PackedTokenStream) -> ParseResult<rlt::IfExpr> {
    tuple((
        match_token(If),
        operator::grammar.cut(),
        block_level::body.cut(),
        many0(elif_expr),
        else_expr.opt(),
    ))
    .context(function!())
    .map(|it| rlt::IfExpr {
        keyword: Keyword::from_located(it.0),
        condition: it.1,
        body: it.2,
        elif: it.3.into_boxed_slice(),
        el: it.4,
    })
    .parse(input)
}
