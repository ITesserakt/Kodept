use nom::branch::alt;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::{function, match_token, ParseResult};
use crate::lexer::{Identifier::*, Token};
use crate::token_stream::TokenStream;

fn variable_ref(input: TokenStream) -> ParseResult<rlt::Reference> {
    match_token!(Token::Identifier(Identifier(_)))
        .map(|it| rlt::Reference::Identifier(it.span.into()))
        .parse(input)
}

fn type_ref(input: TokenStream) -> ParseResult<rlt::Reference> {
    match_token!(Token::Identifier(Type(_)))
        .map(|it| rlt::Reference::Identifier(it.span.into()))
        .parse(input)
}

pub fn reference(input: TokenStream) -> ParseResult<rlt::Reference> {
    variable_ref.or(type_ref).context(function!()).parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<rlt::Term> {
    alt((reference.map(rlt::Term::Reference),))
        .context(function!())
        .parse(input)
}
