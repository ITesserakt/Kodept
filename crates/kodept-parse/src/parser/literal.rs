use nom::branch::alt;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::{function, match_token, ParseResult};
use crate::lexer::{Literal::*, Token};
use crate::parser::nom::{comma_separated0, paren_enclosed};
use crate::parser::operator;
use crate::token_stream::TokenStream;

fn tuple_literal(input: TokenStream) -> ParseResult<rlt::Literal> {
    paren_enclosed(comma_separated0(operator::grammar))
        .context(function!())
        .map(|it| rlt::Literal::Tuple(it.into()))
        .parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<rlt::Literal> {
    alt((
        match_token!(Token::Literal(Binary(_))).map(|it| rlt::Literal::Binary(it.span)),
        match_token!(Token::Literal(Octal(_))).map(|it| rlt::Literal::Octal(it.span)),
        match_token!(Token::Literal(Hex(_))).map(|it| rlt::Literal::Hex(it.span)),
        match_token!(Token::Literal(Floating(_))).map(|it| rlt::Literal::Floating(it.span)),
        match_token!(Token::Literal(Char(_))).map(|it| rlt::Literal::Char(it.span)),
        match_token!(Token::Literal(String(_))).map(|it| rlt::Literal::String(it.span)),
        tuple_literal,
    ))
    .context(function!())
    .parse(input)
}
