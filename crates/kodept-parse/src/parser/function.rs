use crate::lexer::{Identifier::*, Keyword::*, Symbol::*, Token};
use crate::parser::nom::{comma_separated0, match_token, paren_enclosed};
use crate::parser::parameter::{parameter, typed_parameter};
use crate::parser::{block_level, r#type};
use crate::token_stream::TokenStream;
use crate::{function, OptionTExt};
use crate::{match_token, ParseResult};
use kodept_core::structure::rlt;
use nom::branch::alt;
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

#[allow(unused)]
// TODO
fn abstract_function(input: TokenStream) -> ParseResult<rlt::AbstractFunction> {
    tuple((
        match_token(Abstract),
        match_token(Fun),
        match_token!(Token::Identifier(Identifier(_))),
        paren_enclosed(comma_separated0(typed_parameter)).opt(),
        tuple((match_token(Colon), r#type::grammar)).opt(),
    ))
    .context(function!())
    .map(|it| rlt::AbstractFunction {
        keyword: it.1.span.into(),
        id: it.2.span.into(),
        params: it.3.map_into(),
        return_type: it.4.map(|it| (it.0.span.into(), it.1)),
    })
    .parse(input)
}

pub fn bodied(input: TokenStream) -> ParseResult<rlt::BodiedFunction> {
    tuple((
        match_token(Fun),
        match_token!(Token::Identifier(Identifier(_))),
        paren_enclosed(comma_separated0(parameter)).opt(),
        tuple((match_token(Colon), r#type::grammar.cut())).opt(),
        block_level::body,
    ))
    .context(function!())
    .map(|it| rlt::BodiedFunction {
        keyword: it.0.span.into(),
        id: it.1.span.into(),
        params: it.2.map_into(),
        return_type: it.3.map(|it| (it.0.span.into(), it.1)),
        body: Box::new(it.4),
    })
    .parse(input)
}

#[allow(unused)]
// TODO
pub fn grammar(input: TokenStream) -> ParseResult<rlt::Function> {
    alt((
        abstract_function.map(rlt::Function::Abstract),
        bodied.map(rlt::Function::Bodied),
    ))
    .context(function!())
    .parse(input)
}
