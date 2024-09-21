use nom::branch::alt;
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use crate::common::VerboseEnclosed;
use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::parameter::parameter;
use crate::nom::parser::utils::{comma_separated0, match_token};
use crate::nom::parser::{code_flow, literal, operator, term, ParseResult};
use crate::token_stream::PackedTokenStream;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::Symbol;

fn lambda(input: PackedTokenStream) -> ParseResult<rlt::Expression> {
    tuple((
        match_token(LBrace),
        comma_separated0(parameter),
        match_token(RBrace),
        match_token(Flow),
        operator::grammar,
    ))
    .context(function!())
    .map(|it| rlt::Expression::Lambda {
        binds: VerboseEnclosed::from((it.0, it.1.into_boxed_slice(), it.2)).into(),
        flow: Symbol::from_located(it.3),
        expr: Box::new(it.4),
    })
    .parse(input)
}

pub(super) fn grammar(input: PackedTokenStream) -> ParseResult<rlt::Expression> {
    alt((
        lambda,
        term::grammar.map(rlt::Expression::Term),
        literal::grammar.map(rlt::Expression::Literal),
        code_flow::if_expr.map(|it| rlt::Expression::If(Box::new(it))),
    ))
    .context(function!())
    .parse(input)
}
