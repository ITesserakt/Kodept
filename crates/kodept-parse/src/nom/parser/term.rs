use std::collections::VecDeque;

use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::{Context, ContextualReference};

use crate::lexer::Symbol::DoubleColon;
use crate::lexer::{Identifier::*, Token};
use crate::nom::parser::macros::{function, match_token};
use crate::nom::parser::utils::match_token;
use crate::nom::parser::ParseResult;
use crate::token_stream::TokenStream;

/// |      | Global   | Local     |
/// | ---- | -------- | --------- |
/// | Type | ::{X::}X | X::X{::X} |
/// | Ref  | ::{X::}x | X::{X::}x |

fn global_type_ref(input: TokenStream) -> ParseResult<(Context, rlt::Reference)> {
    tuple((
        match_token(DoubleColon),
        many0(type_ref.terminated(match_token(DoubleColon))),
        type_ref,
    ))
    .map(|(global, context, ty)| {
        let start = Context::Global {
            colon: global.span.into(),
        };
        let context = context.into_iter().fold(start, |acc, next| Context::Inner {
            parent: Box::new(acc),
            needle: next,
        });
        (context, ty)
    })
    .context(function!())
    .parse(input)
}

fn global_ref(input: TokenStream) -> ParseResult<(Context, rlt::Reference)> {
    tuple((
        match_token(DoubleColon),
        many0(type_ref.terminated(match_token(DoubleColon))),
        variable_ref,
    ))
    .map(|(global, context, r)| {
        let start = Context::Global {
            colon: global.span.into(),
        };
        let context = context.into_iter().fold(start, |acc, next| Context::Inner {
            parent: Box::new(acc),
            needle: next,
        });
        (context, r)
    })
    .context(function!())
    .parse(input)
}

fn local_type_ref(input: TokenStream) -> ParseResult<(Context, rlt::Reference)> {
    tuple((type_ref, many1(match_token(DoubleColon).precedes(type_ref))))
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
        .context(function!())
        .parse(input)
}

fn local_ref(input: TokenStream) -> ParseResult<(Context, rlt::Reference)> {
    tuple((
        many1(type_ref.terminated(match_token(DoubleColon))),
        variable_ref,
    ))
    .map(|(rest, last)| {
        let start = Context::Local;
        let context = rest.into_iter().fold(start, |acc, next| Context::Inner {
            parent: Box::new(acc),
            needle: next,
        });
        (context, last)
    })
    .context(function!())
    .parse(input)
}

fn variable_ref(input: TokenStream) -> ParseResult<rlt::Reference> {
    match_token!(Token::Identifier(Identifier(_)))
        .map(|it| rlt::Reference::Identifier(it.span.into()))
        .context(function!())
        .parse(input)
}

fn type_ref(input: TokenStream) -> ParseResult<rlt::Reference> {
    match_token!(Token::Identifier(Type(_)))
        .map(|it| rlt::Reference::Identifier(it.span.into()))
        .context(function!())
        .parse(input)
}

fn contextual(input: TokenStream) -> ParseResult<ContextualReference> {
    alt((global_type_ref, global_ref, local_ref, local_type_ref))
        .map(|it| ContextualReference {
            context: it.0,
            inner: it.1,
        })
        .context(function!())
        .parse(input)
}

fn reference(input: TokenStream) -> ParseResult<rlt::Reference> {
    variable_ref.or(type_ref).context(function!()).parse(input)
}

pub(super) fn grammar(input: TokenStream) -> ParseResult<rlt::Term> {
    alt((
        contextual.map(rlt::Term::Contextual),
        reference.map(rlt::Term::Reference),
    ))
    .context(function!())
    .parse(input)
}
