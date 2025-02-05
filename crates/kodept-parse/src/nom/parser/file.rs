use nom::branch::alt;
use nom::combinator::{cut, opt};
use nom::error::context;
use nom::multi::{many0, many1};
use nom::sequence::terminated;
use nom::Parser;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{match_any_token, match_token, newline_separated};
use crate::nom::parser::{top_level, PParser};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{Keyword, Symbol, TypeName};

fn module_statement<'t>() -> impl PParser<'t, rlt::Module> {
    context(
        function!(),
        (
            match_token(Module),
            match_token(Type),
            match_token(LBrace),
            newline_separated(top_level::grammar()),
            cut(match_token(RBrace)),
        ),
    )
    .map(|it| rlt::Module::Ordinary {
        keyword: Keyword::from_located(it.0),
        id: TypeName::from_located(it.1),
        lbrace: Symbol::from_located(it.2),
        rest: it.3.into_boxed_slice(),
        rbrace: Symbol::from_located(it.4),
    })
}

fn global_module_statement<'t>() -> impl PParser<'t, rlt::Module> {
    context(
        function!(),
        (
            match_token(Module),
            match_token(Type),
            match_token(Flow),
            cut(many0(top_level::grammar())),
        ),
    )
    .map(|it| rlt::Module::Global {
        keyword: Keyword::from_located(it.0),
        id: TypeName::from_located(it.1),
        flow: Symbol::from_located(it.2),
        rest: it.3.into_boxed_slice(),
    })
}

pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::File> {
    let modules_parser = many1(module_statement())
        .map(|m| rlt::File::new(m.into_boxed_slice()))
        .or(global_module_statement().map(|m| rlt::File::new(Box::new([m]))));

    context(
        function!(),
        terminated(
            modules_parser,
            opt(alt((
                match_any_token(Comment),
                match_any_token(Newline),
                match_any_token(MultilineComment),
                match_any_token(Whitespace),
            ))),
        ),
    )
}
