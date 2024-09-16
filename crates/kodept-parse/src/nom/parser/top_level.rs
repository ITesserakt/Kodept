use nom::branch::alt;
use nom::combinator::cut;
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::TopLevelNode;

use crate::lexer::{Keyword::*, Symbol::*};
use crate::nom::parser::macros::function;
use crate::nom::parser::parameter::typed_parameter;
use crate::nom::parser::utils::{brace_enclosed, comma_separated0, comma_separated1, match_token, newline_separated, paren_enclosed};
use crate::nom::parser::{function, r#type, ParseResult};
use crate::token_stream::TokenStream;

fn enum_statement(input: TokenStream) -> ParseResult<rlt::Enum> {
    tuple((
        match_token(Enum),
        match_token(Struct).or(match_token(Class)).cut(),
        r#type::reference,
        cut(alt((
            match_token(Semicolon).value(None),
            brace_enclosed(comma_separated1(r#type::reference)).map(Some),
        ))),
    ))
    .context(function!())
    .map(|it| rlt::Enum::Stack {
        keyword: it.0.span.into(),
        id: it.2,
        contents: it.3.map(|it| it.into()),
    })
    .parse(input)
}

fn struct_statement(input: TokenStream) -> ParseResult<rlt::Struct> {
    tuple((
        match_token(Struct),
        r#type::reference.cut(),
        paren_enclosed(comma_separated0(typed_parameter)).opt(),
        brace_enclosed(newline_separated(function::bodied)).opt(),
    ))
    .context(function!())
    .map(|it| rlt::Struct {
        keyword: it.0.span.into(),
        id: it.1,
        parameters: it.2.map(|it| it.into()),
        body: it.3.map(|it| it.into()),
    })
    .parse(input)
}

pub(super) fn grammar(input: TokenStream) -> ParseResult<TopLevelNode> {
    alt((
        enum_statement.map(TopLevelNode::Enum),
        struct_statement.map(TopLevelNode::Struct),
        function::bodied.map(TopLevelNode::BodiedFunction),
    ))
    .context(function!())
    .parse(input)
}
