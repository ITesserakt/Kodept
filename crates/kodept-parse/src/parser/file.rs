use nom::multi::{many0, many1};
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::parser::nom::{match_token, newline_separated};
use crate::parser::top_level;
use crate::{
    function,
    lexer::{Identifier::*, Keyword::*, Operator::*, Symbol::*, Token},
    match_token,
    token_stream::TokenStream,
};
use crate::{match_any_token, ParseResult};

fn module_statement(input: TokenStream) -> ParseResult<rlt::Module> {
    tuple((
        match_token(Module),
        match_token!(Token::Identifier(Type(_))),
        match_token(LBrace),
        newline_separated(top_level::grammar),
        match_token(RBrace).cut(),
    ))
    .context(function!())
    .map(|it| rlt::Module::Ordinary {
        keyword: it.0.span.into(),
        id: it.1.span.into(),
        lbrace: it.2.span.into(),
        rest: it.3.into_boxed_slice(),
        rbrace: it.4.span.into(),
    })
    .parse(input)
}

fn global_module_statement(input: TokenStream) -> ParseResult<rlt::Module> {
    tuple((
        match_token(Module),
        match_token!(Token::Identifier(Type(_))),
        match_token(Flow),
        many0(top_level::grammar).cut(),
    ))
    .context(function!())
    .map(|it| rlt::Module::Global {
        keyword: it.0.span.into(),
        id: it.1.span.into(),
        flow: it.2.span.into(),
        rest: it.3.into_boxed_slice(),
    })
    .parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<rlt::File> {
    many1(module_statement)
        .map(|m| rlt::File::new(m.into_boxed_slice()))
        .or(global_module_statement.map(|m| rlt::File::new(Box::new([m]))))
        .terminated(match_any_token!(Token::Ignore(_)).opt())
        .context(function!())
        .parse(input)
}
