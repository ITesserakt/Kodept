use nom::combinator::{cut, opt};
use nom::error::context;
use nom::Parser;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::parameter::{parameter, typed_parameter};
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::{block_level, r#type, PParser};
use kodept_rlt::prelude as rlt;
use kodept_rlt::new_types;
use kodept_rlt::new_types::{Keyword, Symbol};

#[allow(unused)]
// TODO
fn abstract_function<'t>() -> impl PParser<'t, rlt::AbstractFunction> {
    context(
        function!(),
        (
            match_token(Abstract),
            match_token(Fun),
            match_token(Identifier),
            opt(paren_enclosed(comma_separated0(typed_parameter()))),
            opt((match_token(Colon), r#type::grammar())),
        ),
    )
    .map(|it| rlt::AbstractFunction {
        keyword: Keyword::from_located(it.1),
        id: new_types::Identifier::from_located(it.2),
        params: it.3.map(|it| it.into()),
        return_type: it.4.map(|it| (Symbol::from_located(it.0), it.1)),
    })
}

pub(super) fn bodied<'t>() -> impl PParser<'t, rlt::BodiedFunction> {
    context(
        function!(),
        (
            match_token(Fun),
            match_token(Identifier),
            opt(paren_enclosed(comma_separated0(parameter()))),
            opt((match_token(Colon), cut(r#type::grammar()))),
            cut(block_level::body()),
        ),
    )
    .map(|it| rlt::BodiedFunction {
        keyword: Keyword::from_located(it.0),
        id: new_types::Identifier::from_located(it.1),
        params: it.2.map(|it| it.into()),
        return_type: it.3.map(|it| (Symbol::from_located(it.0), it.1)),
        body: Box::new(it.4),
    })
}
