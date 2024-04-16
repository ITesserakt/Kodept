use nom::branch::alt;
use nom::multi::separated_list0;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::TypeName;

use crate::{function, match_token, ParseResult};
use crate::lexer::{BitOperator::OrBit, Identifier::Type, Operator::Bit, Token};
use crate::parser::nom::{comma_separated0, match_token, paren_enclosed};
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

fn union(input: TokenStream) -> ParseResult<rlt::Type> {
    paren_enclosed(separated_list0(match_token(Bit(OrBit)), grammar))
        .context(function!())
        .map(|it| rlt::Type::Union(it.into()))
        .parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<rlt::Type> {
    alt((reference.map(rlt::Type::Reference), tuple, union))
        .context(function!())
        .parse(input)
}
