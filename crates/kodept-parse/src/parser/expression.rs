use nom::branch::alt;
use nom::Parser;
use nom::sequence::tuple;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::{function, ParseResult};
use crate::lexer::Keyword::Lambda;
use crate::lexer::Operator::Flow;
use crate::parser::{code_flow, literal, operator, term};
use crate::parser::nom::{comma_separated0, match_token};
use crate::token_stream::TokenStream;

fn lambda(input: TokenStream) -> ParseResult<rlt::Expression> {
    tuple((
        match_token(Lambda),
        comma_separated0(term::reference),
        match_token(Flow),
        operator::grammar,
    ))
    .context(function!())
    .map(|it| rlt::Expression::Lambda {
        keyword: it.0.span.into(),
        binds: it.1.into_boxed_slice(),
        flow: it.2.span.into(),
        expr: Box::new(it.3),
    })
    .parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<rlt::Expression> {
    alt((
        lambda,
        term::grammar.map(rlt::Expression::Term),
        literal::grammar.map(rlt::Expression::Literal),
        code_flow::if_expr.map(|it| rlt::Expression::If(Box::new(it))),
    ))
    .context(function!())
    .parse(input)
}
