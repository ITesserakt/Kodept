use nom::branch::alt;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::TypeName;

use crate::lexer::{Identifier::Type, Token};
use crate::nom::parser::macros::{function, match_token};
use crate::nom::parser::ParseResult;
use crate::nom::parser::utils::{comma_separated0, paren_enclosed};
use crate::token_stream::TokenStream;

pub fn reference(input: TokenStream) -> ParseResult<TypeName> {
    match_token!(Token::Identifier(Type(_)))
        .context(function!())
        .map(|it| it.span.into())
        .parse(input)
}

fn tuple(input: TokenStream) -> ParseResult<rlt::Type> {
    paren_enclosed(comma_separated0(grammar))
        .context(function!())
        .map(|it| rlt::Type::Tuple(it.into()))
        .parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<rlt::Type> {
    alt((reference.map(rlt::Type::Reference), tuple))
        .context(function!())
        .parse(input)
}
