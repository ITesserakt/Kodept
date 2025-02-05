use nom::branch::alt;
use nom::error::context;
use nom::Parser;

use kodept_rlt::new_types::Symbol;
use kodept_rlt::prelude as rlt;

use crate::common::VerboseEnclosed;
use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::parameter::parameter;
use crate::nom::parser::utils::{comma_separated0, match_token};
use crate::nom::parser::{code_flow, literal, operator, term, PParser};

fn lambda<'t>() -> impl PParser<'t, rlt::Expression> {
    context(
        function!(),
        (
            match_token(LBrace),
            comma_separated0(parameter()),
            match_token(RBrace),
            match_token(Flow),
            operator::grammar(),
        ),
    )
    .map(|it| {
        rlt::Expression::Lambda(rlt::Lambda {
            binds: VerboseEnclosed::from((it.0, it.1.into_boxed_slice(), it.2)).into(),
            flow: Symbol::from_located(it.3),
            expr: Box::new(it.4),
        })
    })
}

pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::Expression> {
    context(
        function!(),
        alt((
            lambda(),
            term::grammar().map(rlt::Expression::Term),
            literal::grammar().map(rlt::Expression::Literal),
            code_flow::if_expr().map(|it| rlt::Expression::If(Box::new(it))),
        )),
    )
}
