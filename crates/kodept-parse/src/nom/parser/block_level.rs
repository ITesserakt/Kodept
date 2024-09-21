use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::{function, match_token};
use crate::nom::parser::utils::{match_token, newline_separated};
use crate::nom::parser::{function, operator, r#type, ParseResult};
use crate::token_stream::PackedTokenStream;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types;
use kodept_core::structure::rlt::new_types::{Keyword, Symbol};
use nom::branch::alt;
use nom::sequence::tuple;
use nom::Parser;
use nom_supreme::ParserExt;

fn block(input: PackedTokenStream) -> ParseResult<rlt::ExpressionBlock> {
    tuple((
        match_token(LBrace),
        newline_separated(grammar),
        match_token(RBrace),
    ))
    .context(function!())
    .map(|it| rlt::ExpressionBlock {
        lbrace: Symbol::from_located(it.0),
        expression: it.1.into_boxed_slice(),
        rbrace: Symbol::from_located(it.2),
    })
    .parse(input)
}

fn simple(input: PackedTokenStream) -> ParseResult<rlt::Body> {
    tuple((match_token(Flow), grammar.cut()))
        .context(function!())
        .map(|it| rlt::Body::Simplified {
            flow: Symbol::from_located(it.0),
            expression: it.1,
        })
        .parse(input)
}

pub(super) fn body(input: PackedTokenStream) -> ParseResult<rlt::Body> {
    alt((block.map(rlt::Body::Block), simple))
        .context(function!())
        .parse(input)
}

#[allow(unused_parens)]
fn var_declaration(input: PackedTokenStream) -> ParseResult<rlt::Variable> {
    let (input, kind) = match_token(Val).or(match_token(Var)).parse(input)?;
    let (input, rest) = tuple((
        match_token!(Identifier).cut(),
        tuple((match_token(Colon), r#type::grammar)).opt(),
    ))
    .cut()
    .context(function!())
    .parse(input)?;

    if kind.token == Val {
        Ok((
            input,
            rlt::Variable::Immutable {
                keyword: Keyword::from_located(kind),
                id: new_types::Identifier::from_located(rest.0),
                assigned_type: rest.1.map(|it| (Symbol::from_located(it.0), it.1)),
            },
        ))
    } else {
        Ok((
            input,
            rlt::Variable::Mutable {
                keyword: Keyword::from_located(kind),
                id: new_types::Identifier::from_located(rest.0),
                assigned_type: rest.1.map(|it| (Symbol::from_located(it.0), it.1)),
            },
        ))
    }
}

fn initialized_variable(input: PackedTokenStream) -> ParseResult<rlt::InitializedVariable> {
    tuple((
        var_declaration,
        match_token(Equals).cut(),
        operator::grammar,
    ))
    .context(function!())
    .map(|it| rlt::InitializedVariable {
        variable: it.0,
        equals: Symbol::from_located(it.1),
        expression: it.2,
    })
    .parse(input)
}

pub(super) fn grammar(input: PackedTokenStream) -> ParseResult<rlt::BlockLevelNode> {
    alt((
        block.map(rlt::BlockLevelNode::Block),
        initialized_variable.map(rlt::BlockLevelNode::InitVar),
        function::bodied.map(rlt::BlockLevelNode::Function),
        operator::grammar.map(rlt::BlockLevelNode::Operation),
    ))
    .context(function!())
    .parse(input)
}
