use crate::lexer::{Keyword::*, Symbol::*};
use crate::parser::nom::{
    brace_enclosed, comma_separated0, comma_separated1, match_token, newline_separated,
    paren_enclosed,
};
use crate::parser::parameter::typed_parameter;
use crate::parser::{function, r#type};
use crate::token_stream::TokenStream;
use crate::ParseResult;
use crate::{function, OptionTExt};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::TopLevelNode;
use nom::branch::alt;
use nom::combinator::cut;
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

fn enum_statement(input: TokenStream) -> ParseResult<rlt::Enum> {
    tuple((
        match_token(Enum),
        match_token(Struct).or(match_token(Class)),
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
        contents: it.3.map_into(),
    })
    .parse(input)
}

fn struct_statement(input: TokenStream) -> ParseResult<rlt::Struct> {
    tuple((
        match_token(Struct),
        r#type::reference,
        paren_enclosed(comma_separated0(typed_parameter)).opt(),
        brace_enclosed(newline_separated(function::bodied)).opt(),
    ))
    .context(function!())
    .map(|it| rlt::Struct {
        keyword: it.0.span.into(),
        id: it.1,
        parameters: it.2.map_into(),
        body: it.3.map_into(),
    })
    .parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<TopLevelNode> {
    alt((
        enum_statement.map(TopLevelNode::Enum),
        struct_statement.map(TopLevelNode::Struct),
        function::bodied.map(TopLevelNode::BodiedFunction),
    ))
    .context(function!())
    .parse(input)
}
