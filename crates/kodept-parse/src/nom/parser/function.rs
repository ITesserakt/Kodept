use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::lexer::{Identifier::*, Keyword::*, Symbol::*, Token};
use crate::nom::parser::macros::{function, match_token};
use crate::nom::parser::parameter::{parameter, typed_parameter};
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::{block_level, r#type, ParseResult};
use crate::token_stream::TokenStream;

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
        params: it.3.map(|it| it.into()),
        return_type: it.4.map(|it| (it.0.span.into(), it.1)),
    })
    .parse(input)
}

pub(super) fn bodied(input: TokenStream) -> ParseResult<rlt::BodiedFunction> {
    tuple((
        match_token(Fun),
        match_token!(Token::Identifier(Identifier(_))),
        paren_enclosed(comma_separated0(parameter)).opt(),
        tuple((match_token(Colon), r#type::grammar.cut())).opt(),
        block_level::body.cut(),
    ))
    .context(function!())
    .map(|it| rlt::BodiedFunction {
        keyword: it.0.span.into(),
        id: it.1.span.into(),
        params: it.2.map(|it| it.into()),
        return_type: it.3.map(|it| (it.0.span.into(), it.1)),
        body: Box::new(it.4),
    })
    .parse(input)
}
