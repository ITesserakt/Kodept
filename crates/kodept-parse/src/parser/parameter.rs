use crate::lexer::{
    Identifier::Identifier,
    Symbol::{Colon, TypeGap},
    Token,
};
use crate::parser::r#type;
use crate::token_stream::TokenStream;
use crate::{function, match_token, ParseResult};
use kodept_core::structure::rlt;
use nom::branch::alt;
use nom::sequence::separated_pair;
use nom::Parser;
use nom_supreme::ParserExt;

pub fn typed_parameter(input: TokenStream) -> ParseResult<rlt::TypedParameter> {
    separated_pair(
        match_token!(Token::Identifier(Identifier(_))),
        match_token!(Token::Symbol(Colon)),
        r#type::grammar,
    )
    .context(function!())
    .map(|it| rlt::TypedParameter {
        id: it.0.span.into(),
        parameter_type: it.1,
    })
    .parse(input)
}

pub fn untyped_parameter(input: TokenStream) -> ParseResult<rlt::UntypedParameter> {
    let (rest, id) = match_token!(Token::Identifier(Identifier(_)))
        .context(function!())
        .parse(input)?;
    let (rest, _) = match_token!(Token::Symbol(Colon))
        .precedes(match_token!(Token::Symbol(TypeGap)).cut())
        .opt()
        .parse(rest)?;

    Ok((rest, rlt::UntypedParameter { id: id.span.into() }))
}

pub fn parameter(input: TokenStream) -> ParseResult<rlt::Parameter> {
    alt((
        typed_parameter.map(rlt::Parameter::Typed),
        untyped_parameter.map(rlt::Parameter::Untyped),
    ))
    .context(function!())
    .parse(input)
}
