//! |      | Global   | Local     |
//! | ---- | -------- | --------- |
//! | Type | ::{X::}X | X::X{::X} |
//! | Ref  | ::{X::}x | X::{X::}x |

use std::collections::VecDeque;

use nom::branch::alt;
use nom::error::context;
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated};
use nom::Parser;

use kodept_rlt::new_types::{Identifier, Symbol, TypeName};
use kodept_rlt::prelude as rlt;
use kodept_rlt::prelude::{Context, Contextual};

use crate::lexer::PackedToken::{self, *};
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::match_token;
use crate::nom::parser::PParser;

fn global_type_ref<'t>() -> impl PParser<'t, (Context, TypeName)> {
    context(
        function!(),
        (
            match_token(DoubleColon),
            many0(terminated(type_ref(), match_token(DoubleColon))),
            type_ref(),
        ),
    )
    .map(|(global, context, ty)| {
        let start = Context::Global {
            colon: Symbol::from_located(global),
        };
        let context = context.into_iter().fold(start, |acc, next| Context::Inner {
            parent: Box::new(acc),
            needle: next,
        });
        (context, ty)
    })
}

fn global_ref<'t>() -> impl PParser<'t, (Context, Identifier)> {
    context(
        function!(),
        (
            match_token(DoubleColon),
            many0(terminated(type_ref(), match_token(DoubleColon))),
            variable_ref(),
        ),
    )
    .map(|(global, context, r)| {
        let start = Context::Global {
            colon: Symbol::from_located(global),
        };
        let context = context.into_iter().fold(start, |acc, next| Context::Inner {
            parent: Box::new(acc),
            needle: next,
        });
        (context, r)
    })
}

fn local_type_ref<'t>() -> impl PParser<'t, (Context, TypeName)> {
    context(
        function!(),
        (
            type_ref(),
            many1(preceded(match_token(DoubleColon), type_ref())),
        ),
    )
    .map(|it| (it.0, VecDeque::from(it.1)))
    .map(|(first, mut rest)| {
        let start = Context::Local;
        let last = rest
            .pop_back()
            .expect("Used many1 parser, so this is unreachable");
        rest.push_front(first);
        let context = rest.into_iter().fold(start, |acc, next| Context::Inner {
            parent: Box::new(acc),
            needle: next,
        });
        (context, last)
    })
}

fn local_ref<'t>() -> impl PParser<'t, (Context, Identifier)> {
    context(
        function!(),
        (
            many1(terminated(type_ref(), match_token(DoubleColon))),
            variable_ref(),
        ),
    )
    .map(|(rest, last)| {
        let start = Context::Local;
        let context = rest.into_iter().fold(start, |acc, next| Context::Inner {
            parent: Box::new(acc),
            needle: next,
        });
        (context, last)
    })
}

fn variable_ref<'t>() -> impl PParser<'t, Identifier> {
    context(function!(), match_token(PackedToken::Identifier))
        .map(|it| Identifier::from_located(it))
}

fn type_ref<'t>() -> impl PParser<'t, TypeName> {
    context(function!(), match_token(Type)).map(|it| TypeName::from_located(it))
}

fn contextual_variable<'t>() -> impl PParser<'t, Contextual<Identifier>> {
    context(function!(), alt((global_ref(), local_ref()))).map(|it| Contextual {
        context: it.0,
        inner: it.1,
    })
}

fn contextual_type<'t>() -> impl PParser<'t, Contextual<TypeName>> {
    context(function!(), alt((global_type_ref(), local_type_ref()))).map(|it| Contextual {
        context: it.0,
        inner: it.1,
    })
}

pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::Term> {
    context(
        function!(),
        alt((
            contextual_variable().map(rlt::Term::ContextualReference),
            contextual_type().map(rlt::Term::ContextualConstant),
            variable_ref().map(rlt::Term::Reference),
            type_ref().map(rlt::Term::Constant),
        )),
    )
}
