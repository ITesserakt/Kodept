use nom::branch::alt;
use nom::multi::many0;
use nom::Parser;
use nom::sequence::{delimited, tuple};
use nom_supreme::ParserExt;
use nonempty_collections::NEVec;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{
    BinaryOperationSymbol, Enclosed, Symbol, UnaryOperationSymbol,
};

use crate::{function, ParseResult};
use crate::lexer::BitOperator::{AndBit, NotBit, OrBit, XorBit};
use crate::lexer::ComparisonOperator::{
    Equiv, Greater, GreaterEquals, Less, LessEquals, NotEquiv, Spaceship,
};
use crate::lexer::LogicOperator::{AndLogic, NotLogic, OrLogic};
use crate::lexer::MathOperator::{Div, Mod, Plus, Pow, Sub, Times};
use crate::lexer::Operator::{Bit, Comparison, Dot, Logic, Math};
use crate::lexer::Symbol::{LParen, RParen};
use crate::parser::expression;
use crate::parser::nom::{comma_separated0, match_token, paren_enclosed};
use crate::token_match::TokenMatch;
use crate::token_stream::TokenStream;

fn left_fold<'t, I, T, P, E, F, R>(parser: P, produce: F) -> impl Parser<I, R, E> + 't
where
    P: Parser<I, (T, Vec<(TokenMatch<'t>, T)>), E> + 't,
    F: Fn(R, Symbol, T) -> R + 'static,
    T: 't,
    R: From<T>,
{
    parser.map(move |(a, tail)| match NEVec::from_vec(tail) {
        None => a.into(),
        Some(rest) => {
            let (op, b) = rest.head;
            rest.tail
                .into_iter()
                .fold(produce(a.into(), op.span.into(), b), |a, (op, b)| {
                    produce(a, op.span.into(), b)
                })
        }
    })
}

fn right_fold<'t, I, T, P, E, R, F>(parser: P, produce: F) -> impl Parser<I, R, E> + 't
where
    P: Parser<I, (T, Option<(TokenMatch<'t>, T)>), E> + 't,
    R: From<T>,
    F: Fn(R, Symbol, T) -> R + 'static,
    T: 't,
{
    parser.map(move |(a, tail)| match tail {
        None => a.into(),
        Some((op, b)) => produce(a.into(), op.span.into(), b),
    })
}

fn atom(input: TokenStream) -> ParseResult<rlt::Operation> {
    alt((
        delimited(match_token(LParen), grammar, match_token(RParen)),
        expression::grammar.map(rlt::Operation::Expression),
    ))
    .context(function!())
    .parse(input)
}

fn access(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        tuple((atom, many0(tuple((match_token(Dot), atom))))),
        |a, op, b| rlt::Operation::Access {
            left: Box::new(a),
            dot: op,
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn top_expr(input: TokenStream) -> ParseResult<rlt::Operation> {
    alt((
        match_token(Math(Sub)).map(|it| UnaryOperationSymbol::Neg(it.span.into())),
        match_token(Logic(NotLogic)).map(|it| UnaryOperationSymbol::Not(it.span.into())),
        match_token(Bit(NotBit)).map(|it| UnaryOperationSymbol::Inv(it.span.into())),
        match_token(Math(Plus)).map(|it| UnaryOperationSymbol::Plus(it.span.into())),
    ))
    .and(top_expr)
    .map(|it| rlt::Operation::TopUnary {
        operator: it.0,
        expr: Box::new(it.1),
    })
    .or(access)
    .context(function!())
    .parse(input)
}

fn pow_expr(input: TokenStream) -> ParseResult<rlt::Operation> {
    right_fold(
        top_expr.and(match_token(Math(Pow)).and(pow_expr).opt()),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Pow(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn mul_expr(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        pow_expr.and(many0(
            alt((
                match_token(Math(Times)),
                match_token(Math(Div)),
                match_token(Math(Mod)),
            ))
            .and(pow_expr),
        )),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Mul(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn add_expr(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        mul_expr.and(many0(
            alt((match_token(Math(Plus)), match_token(Math(Sub)))).and(mul_expr),
        )),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Add(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn complex_cmp(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        add_expr.and(many0(match_token(Comparison(Spaceship)).and(add_expr))),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::ComplexComparison(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn compound_cmp(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        complex_cmp.and(many0(
            alt((
                match_token(Comparison(LessEquals)),
                match_token(Comparison(NotEquiv)),
                match_token(Comparison(Equiv)),
                match_token(Comparison(GreaterEquals)),
            ))
            .and(complex_cmp),
        )),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::CompoundComparison(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn simple_cmp(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        compound_cmp.and(many0(
            alt((
                match_token(Comparison(Less)),
                match_token(Comparison(Greater)),
            ))
            .and(compound_cmp),
        )),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Comparison(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn bit_expr(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        simple_cmp.and(many0(
            alt((
                match_token(Bit(OrBit)),
                match_token(Bit(AndBit)),
                match_token(Bit(XorBit)),
            ))
            .and(simple_cmp),
        )),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Bit(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn logic_expr(input: TokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        bit_expr.and(many0(
            alt((match_token(Logic(OrLogic)), match_token(Logic(AndLogic)))).and(bit_expr),
        )),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Logic(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn parameters(input: TokenStream) -> ParseResult<Enclosed<Box<[rlt::Operation]>>> {
    paren_enclosed(comma_separated0(application))
        .context(function!())
        .map(|it| it.into())
        .parse(input)
}

fn application(input: TokenStream) -> ParseResult<rlt::Operation> {
    tuple((logic_expr, parameters.opt()))
        .context(function!())
        .map(|(expr, params)| match params {
            None => expr,
            Some(_) => rlt::Operation::Application(Box::new(rlt::Application { expr, params })),
        })
        .parse(input)
}

#[inline]
pub fn grammar(input: TokenStream) -> ParseResult<rlt::Operation> {
    application(input)
}
