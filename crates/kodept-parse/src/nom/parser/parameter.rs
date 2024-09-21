use nom::branch::alt;
use nom::sequence::separated_pair;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types;
use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::{function};
use crate::nom::parser::{r#type, ParseResult};
use crate::nom::parser::utils::match_token;
use crate::token_stream::PackedTokenStream;

pub(super) fn typed_parameter(input: PackedTokenStream) -> ParseResult<rlt::TypedParameter> {
    separated_pair(
        match_token(Identifier),
        match_token(Colon),
        r#type::grammar,
    )
    .context(function!())
    .map(|it| rlt::TypedParameter {
        id: new_types::Identifier::from_located(it.0),
        parameter_type: it.1,
    })
    .parse(input)
}

fn untyped_parameter(input: PackedTokenStream) -> ParseResult<rlt::UntypedParameter> {
    let (rest, id) = match_token(Identifier).context(function!()).parse(input)?;
    let (rest, _) = match_token(Colon)
        .precedes(match_token(TypeGap).cut())
        .opt()
        .parse(rest)?;

    Ok((rest, rlt::UntypedParameter { id: new_types::Identifier::from_located(id) }))
}

pub(super) fn parameter(input: PackedTokenStream) -> ParseResult<rlt::Parameter> {
    alt((
        typed_parameter.map(rlt::Parameter::Typed),
        untyped_parameter.map(rlt::Parameter::Untyped),
    ))
    .context(function!())
    .parse(input)
}
