use nom::branch::alt;
use nom::error::context;
use nom::Parser;

use kodept_rlt::new_types::TypeName;
use kodept_rlt::prelude as rlt;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::PParser;

pub(super) fn reference<'t>() -> impl PParser<'t, TypeName> {
    context(function!(), match_token(Type)).map(TypeName::from_located)
}

fn tuple<'t>() -> impl PParser<'t, rlt::Type> {
    context(function!(), paren_enclosed(comma_separated0(grammar())))
        .map(|it| rlt::Type::Tuple(rlt::Tuple(it.into())))
}

pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::Type> {
    context(function!(), |input| {
        alt((reference().map(rlt::Type::Reference), tuple())).parse(input)
    })
}
