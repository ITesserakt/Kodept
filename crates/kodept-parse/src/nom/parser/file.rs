use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{match_any_token, match_token, newline_separated};
use crate::nom::parser::{top_level, ParseResult};
use crate::token_stream::PackedTokenStream;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{Keyword, Symbol, TypeName};

fn module_statement(input: PackedTokenStream) -> ParseResult<rlt::Module> {
    tuple((
        match_token(Module),
        match_token(Type),
        match_token(LBrace),
        newline_separated(top_level::grammar),
        match_token(RBrace).cut(),
    ))
    .context(function!())
    .map(|it| rlt::Module::Ordinary {
        keyword: Keyword::from_located(it.0),
        id: TypeName::from_located(it.1),
        lbrace: Symbol::from_located(it.2),
        rest: it.3.into_boxed_slice(),
        rbrace: Symbol::from_located(it.4),
    })
    .parse(input)
}

fn global_module_statement(input: PackedTokenStream) -> ParseResult<rlt::Module> {
    tuple((
        match_token(Module),
        match_token(Type),
        match_token(Flow),
        many0(top_level::grammar).cut(),
    ))
    .context(function!())
    .map(|it| rlt::Module::Global {
        keyword: Keyword::from_located(it.0),
        id: TypeName::from_located(it.1),
        flow: Symbol::from_located(it.2),
        rest: it.3.into_boxed_slice(),
    })
    .parse(input)
}

#[allow(unused_parens)]
pub(super) fn grammar(input: PackedTokenStream) -> ParseResult<rlt::File> {
    many1(module_statement)
        .map(|m| rlt::File::new(m.into_boxed_slice()))
        .or(global_module_statement.map(|m| rlt::File::new(Box::new([m]))))
        .terminated(
            alt((
                match_any_token(Comment),
                match_any_token(Newline),
                match_any_token(MultilineComment),
                match_any_token(Whitespace),
            ))
            .opt(),
        )
        .context(function!())
        .parse(input)
}
