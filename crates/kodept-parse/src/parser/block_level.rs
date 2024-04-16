use nom::branch::alt;
use nom::Parser;
use nom::sequence::tuple;
use nom_supreme::ParserExt;

use kodept_core::structure::rlt;

use crate::{function, match_token, ParseResult};
use crate::lexer::{
    ComparisonOperator::Equals,
    Identifier::Identifier,
    Keyword::*,
    Operator::{Comparison, Flow},
    Symbol::*,
    Token,
};
use crate::parser::{function, operator, r#type};
use crate::parser::nom::{match_token, newline_separated};
use crate::token_stream::TokenStream;

pub fn block(input: TokenStream) -> ParseResult<rlt::ExpressionBlock> {
    tuple((
        match_token(LBrace),
        newline_separated(grammar),
        match_token(RBrace),
    ))
    .context(function!())
    .map(|it| rlt::ExpressionBlock {
        lbrace: it.0.span.into(),
        expression: it.1.into_boxed_slice(),
        rbrace: it.2.span.into(),
    })
    .parse(input)
}

pub fn simple(input: TokenStream) -> ParseResult<rlt::Body> {
    tuple((match_token(Flow), grammar.cut()))
        .context(function!())
        .map(|it| rlt::Body::Simplified {
            flow: it.0.span.into(),
            expression: it.1,
        })
        .parse(input)
}

pub fn body(input: TokenStream) -> ParseResult<rlt::Body> {
    alt((block.map(rlt::Body::Block), simple))
        .context(function!())
        .parse(input)
}

#[allow(unused_parens)]
fn var_declaration(input: TokenStream) -> ParseResult<rlt::Variable> {
    let (input, kind) = match_token(Val).or(match_token(Var)).parse(input)?;
    let (input, rest) = tuple((
        match_token!(Token::Identifier(Identifier(_))).cut(),
        tuple((match_token(Colon), r#type::grammar)).opt(),
    ))
    .cut()
    .context(function!())
    .parse(input)?;

    if kind.token == Token::Keyword(Val) {
        Ok((
            input,
            rlt::Variable::Immutable {
                keyword: kind.span.into(),
                id: rest.0.span.into(),
                assigned_type: rest.1.map(|it| (it.0.span.into(), it.1)),
            },
        ))
    } else {
        Ok((
            input,
            rlt::Variable::Mutable {
                keyword: kind.span.into(),
                id: rest.0.span.into(),
                assigned_type: rest.1.map(|it| (it.0.span.into(), it.1)),
            },
        ))
    }
}

fn initialized_variable(input: TokenStream) -> ParseResult<rlt::InitializedVariable> {
    tuple((
        var_declaration,
        match_token(Comparison(Equals)).cut(),
        operator::grammar,
    ))
    .context(function!())
    .map(|it| rlt::InitializedVariable {
        variable: it.0,
        equals: it.1.span.into(),
        expression: it.2,
    })
    .parse(input)
}

pub fn grammar(input: TokenStream) -> ParseResult<rlt::BlockLevelNode> {
    alt((
        block.map(rlt::BlockLevelNode::Block),
        initialized_variable.map(rlt::BlockLevelNode::InitVar),
        function::bodied.map(rlt::BlockLevelNode::Function),
        operator::grammar.map(rlt::BlockLevelNode::Operation),
    ))
    .context(function!())
    .parse(input)
}
