use nom::branch::alt;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::TypeName;

use crate::nom::parser::macros::{function};
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::ParseResult;
use crate::token_stream::PackedTokenStream;
use crate::lexer::PackedToken::*;

pub(super) fn reference(input: PackedTokenStream) -> ParseResult<TypeName> {
    match_token(Type)
        .context(function!())
        .map(TypeName::from_located)
        .parse(input)
}

fn tuple(input: PackedTokenStream) -> ParseResult<rlt::Type> {
    paren_enclosed(comma_separated0(grammar))
        .context(function!())
        .map(|it| rlt::Type::Tuple(it.into()))
        .parse(input)
}

pub(super) fn grammar(input: PackedTokenStream) -> ParseResult<rlt::Type> {
    alt((reference.map(rlt::Type::Reference), tuple))
        .context(function!())
        .parse(input)
}
