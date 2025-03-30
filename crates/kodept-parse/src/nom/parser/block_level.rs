use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{match_token, newline_separated};
use crate::nom::parser::{function, operator, r#type, PParser};
use crate::token_stream::PackedTokenStream;
use kodept_rlt::prelude as rlt;
use kodept_rlt::new_types;
use kodept_rlt::new_types::{Keyword, Symbol};
use nom::branch::alt;
use nom::combinator::{cut, map, opt};
use nom::error::context;
use nom::Parser;

fn block<'t>() -> impl PParser<'t, rlt::ExpressionBlock> {
    context(function!(), |input| {
        let (rest, it) = (
            match_token(LBrace),
            newline_separated(grammar()),
            match_token(RBrace),
        )
            .parse(input)?;

        Ok((
            rest,
            rlt::ExpressionBlock {
                lbrace: Symbol::from_located(it.0),
                expression: it.1.into_boxed_slice(),
                rbrace: Symbol::from_located(it.2),
            },
        ))
    })
}

fn simple<'t>() -> impl PParser<'t, rlt::Body> {
    context(
        function!(),
        map((match_token(Flow), cut(grammar())), |it| {
            rlt::Body::Simplified {
                flow: Symbol::from_located(it.0),
                expression: it.1,
            }
        }),
    )
}

pub(super) fn body<'t>() -> impl PParser<'t, rlt::Body> {
    context(function!(), |input| {
        alt((block().map(rlt::Body::Block), simple())).parse(input)
    })
}

#[allow(unused_parens)]
fn var_declaration<'t>() -> impl PParser<'t, rlt::Variable> {
    |input: PackedTokenStream<'t>| {
        let (rest, token_match) = match_token(Val).or(match_token(Var)).parse(input)?;
        let ctor = if token_match.token == Val {
            |keyword, id, assigned_type| rlt::Variable::Immutable {
                keyword,
                id,
                assigned_type,
            }
        } else {
            |keyword, id, assigned_type| rlt::Variable::Mutable {
                keyword,
                id,
                assigned_type,
            }
        };
        let ctor_curried =
            move |id, assigned_type| ctor(Keyword::from_located(token_match), id, assigned_type);

        let mut parser = map(
            context(
                function!(),
                cut((
                    cut(match_token(Identifier)),
                    opt((match_token(Colon), r#type::grammar())),
                )),
            ),
            |it| {
                ctor_curried(
                    new_types::Identifier::from_located(it.0),
                    it.1.map(|it| (Symbol::from_located(it.0), it.1)),
                )
            },
        );
        parser.parse(rest)
    }
}

fn initialized_variable<'t>() -> impl PParser<'t, rlt::InitializedVariable> {
    let parser = context(
        function!(),
        (
            var_declaration(),
            cut(match_token(Equals)),
            operator::grammar(),
        ),
    );

    map(parser, |it| rlt::InitializedVariable {
        variable: it.0,
        equals: Symbol::from_located(it.1),
        expression: it.2,
    })
}

pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::BlockLevelNode> {
    context(
        function!(),
        alt((
            map(block(), rlt::BlockLevelNode::Block),
            map(initialized_variable(), rlt::BlockLevelNode::InitVar),
            map(function::bodied(), rlt::BlockLevelNode::Function),
            map(operator::grammar(), rlt::BlockLevelNode::Operation),
        )),
    )
}
