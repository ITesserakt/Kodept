use nom::branch::alt;
use nom::combinator::{cut, opt};
use nom::error::context;
use nom::sequence::{separated_pair, terminated};
use nom::Parser;

use kodept_rlt::prelude as rlt;
use kodept_rlt::new_types;
use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::match_token;
use crate::nom::parser::{r#type, PParser};

pub(super) fn typed_parameter<'t>() -> impl PParser<'t, rlt::TypedParameter> {
    context(
        function!(),
        separated_pair(match_token(Identifier), match_token(Colon), r#type::grammar()),
    )
    .map(|it| rlt::TypedParameter {
        id: new_types::Identifier::from_located(it.0),
        parameter_type: it.1,
    })
}

fn untyped_parameter<'t>() -> impl PParser<'t, rlt::UntypedParameter> {
    context(
        function!(),
        terminated(
            match_token(Identifier),
            opt((match_token(Colon), cut(match_token(TypeGap)))),
        ),
    )
    .map(|it| rlt::UntypedParameter {
        id: new_types::Identifier::from_located(it),
    })
}

pub(super) fn parameter<'t>() -> impl PParser<'t, rlt::Parameter> {
    context(
        function!(),
        alt((
            typed_parameter().map(rlt::Parameter::Typed),
            untyped_parameter().map(rlt::Parameter::Untyped),
        )),
    )
}
