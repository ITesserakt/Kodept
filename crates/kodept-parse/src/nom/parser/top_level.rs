use nom::branch::alt;
use nom::combinator::{cut, opt, recognize, value};
use nom::error::context;
use nom::Parser;

use kodept_rlt::prelude as rlt;
use kodept_rlt::new_types::Keyword;
use kodept_rlt::prelude::TopLevelNode;
use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::parameter::typed_parameter;
use crate::nom::parser::utils::{
    brace_enclosed, comma_separated0, comma_separated1, match_token, newline_separated,
    paren_enclosed,
};
use crate::nom::parser::{function, r#type, PParser};

fn enum_statement<'t>() -> impl PParser<'t, rlt::Enum> {
    context(
        function!(),
        (
            recognize((
                match_token(Enum),
                cut(match_token(Struct).or(match_token(Class))),
            )),
            r#type::reference(),
            alt((
                value(None, match_token(Semicolon)),
                brace_enclosed(comma_separated1(r#type::reference())).map(Some),
            )),
        ),
    )
    .map(|it| rlt::Enum::Stack {
        keyword: Keyword::from_located(it.0),
        id: it.1,
        contents: it.2.map(|it| it.into()),
    })
}

fn struct_statement<'t>() -> impl PParser<'t, rlt::Struct> {
    context(
        function!(),
        (
            match_token(Struct),
            cut(r#type::reference()),
            opt(paren_enclosed(comma_separated0(typed_parameter()))),
            opt(brace_enclosed(newline_separated(function::bodied()))),
        ),
    )
    .map(|it| rlt::Struct {
        keyword: Keyword::from_located(it.0),
        id: it.1,
        parameters: it.2.map(|it| it.into()),
        body: it.3.map(|it| it.into()),
    })
}

pub(super) fn grammar<'t>() -> impl PParser<'t, TopLevelNode> {
    alt((
        enum_statement().map(TopLevelNode::Enum),
        struct_statement().map(TopLevelNode::Struct),
        function::bodied().map(TopLevelNode::BodiedFunction),
    ))
}
