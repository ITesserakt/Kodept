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

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::match_token;
use crate::nom::parser::PParser;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::Symbol;
use kodept_core::structure::rlt::{new_types, Context, ContextualReference};

fn global_type_ref<'t>() -> impl PParser<'t, (Context, rlt::Reference)> {
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

fn global_ref<'t>() -> impl PParser<'t, (Context, rlt::Reference)> {
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

fn local_type_ref<'t>() -> impl PParser<'t, (Context, rlt::Reference)> {
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

fn local_ref<'t>() -> impl PParser<'t, (Context, rlt::Reference)> {
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

fn variable_ref<'t>() -> impl PParser<'t, rlt::Reference> {
    context(function!(), match_token(Identifier))
        .map(|it| rlt::Reference::Identifier(new_types::Identifier::from_located(it)))
}

fn type_ref<'t>() -> impl PParser<'t, rlt::Reference> {
    context(function!(), match_token(Type))
        .map(|it| rlt::Reference::Identifier(new_types::Identifier::from_located(it)))
}

fn contextual<'t>() -> impl PParser<'t, ContextualReference> {
    context(
        function!(),
        alt((
            global_type_ref(),
            global_ref(),
            local_ref(),
            local_type_ref(),
        )),
    )
    .map(|it| ContextualReference {
        context: it.0,
        inner: it.1,
    })
}

fn reference<'t>() -> impl PParser<'t, rlt::Reference> {
    context(function!(), variable_ref().or(type_ref()))
}

pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::Term> {
    context(
        function!(),
        alt((
            contextual().map(rlt::Term::Contextual),
            reference().map(rlt::Term::Reference),
        )),
    )
}
