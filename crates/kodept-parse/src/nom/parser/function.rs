use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::{function, match_token};
use crate::nom::parser::parameter::{parameter, typed_parameter};
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::{block_level, r#type, ParseResult};
use crate::token_stream::PackedTokenStream;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types;
use kodept_core::structure::rlt::new_types::{Keyword, Symbol};

#[allow(unused)]
// TODO
fn abstract_function(input: PackedTokenStream) -> ParseResult<rlt::AbstractFunction> {
    tuple((
        match_token(Abstract),
        match_token(Fun),
        match_token!(Identifier),
        paren_enclosed(comma_separated0(typed_parameter)).opt(),
        tuple((match_token(Colon), r#type::grammar)).opt(),
    ))
    .context(function!())
    .map(|it| rlt::AbstractFunction {
        keyword: Keyword::from_located(it.1),
        id: new_types::Identifier::from_located(it.2),
        params: it.3.map(|it| it.into()),
        return_type: it.4.map(|it| (Symbol::from_located(it.0), it.1)),
    })
    .parse(input)
}

pub(super) fn bodied(input: PackedTokenStream) -> ParseResult<rlt::BodiedFunction> {
    tuple((
        match_token(Fun),
        match_token!(Identifier),
        paren_enclosed(comma_separated0(parameter)).opt(),
        tuple((match_token(Colon), r#type::grammar.cut())).opt(),
        block_level::body.cut(),
    ))
    .context(function!())
    .map(|it| rlt::BodiedFunction {
        keyword: Keyword::from_located(it.0),
        id: new_types::Identifier::from_located(it.1),
        params: it.2.map(|it| it.into()),
        return_type: it.3.map(|it| (Symbol::from_located(it.0), it.1)),
        body: Box::new(it.4),
    })
    .parse(input)
}
