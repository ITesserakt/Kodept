use nom::branch::alt;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom::Parser;
use nom_supreme::ParserExt;
use nonempty_collections::NEVec;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::{expression, ParseResult};
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{
    BinaryOperationSymbol, Enclosed, Symbol, UnaryOperationSymbol,
};

fn left_fold<I, T, P, E, F, R>(parser: P, produce: F) -> impl Parser<I, R, E>
where
    P: Parser<I, (T, Vec<(PackedTokenMatch, T)>), E>,
    F: Fn(R, Symbol, T) -> R + 'static,
    R: From<T>,
{
    parser.map(move |(a, tail)| match NEVec::from_vec(tail) {
        None => a.into(),
        Some(rest) => {
            let (op, b) = rest.head;
            rest.tail.into_iter().fold(
                produce(a.into(), Symbol::from_located(op), b),
                |a, (op, b)| produce(a, Symbol::from_located(op), b),
            )
        }
    })
}

fn right_fold<I, T, P, E, R, F>(parser: P, produce: F) -> impl Parser<I, R, E>
where
    P: Parser<I, (T, Option<(PackedTokenMatch, T)>), E>,
    R: From<T>,
    F: Fn(R, Symbol, T) -> R + 'static,
{
    parser.map(move |(a, tail)| match tail {
        None => a.into(),
        Some((op, b)) => produce(a.into(), Symbol::from_located(op), b),
    })
}

fn atom(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    alt((
        delimited(match_token(LParen), grammar, match_token(RParen)),
        expression::grammar.map(rlt::Operation::Expression),
    ))
    .context(function!())
    .parse(input)
}

fn access(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
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

fn parameters(input: PackedTokenStream) -> ParseResult<Enclosed<Box<[rlt::Operation]>>> {
    paren_enclosed(comma_separated0(grammar))
        .context(function!())
        .map(|it| it.into())
        .parse(input)
}

fn application(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    tuple((access, parameters.opt()))
        .context(function!())
        .map(|(expr, params)| match params {
            None => expr,
            Some(_) => rlt::Operation::Application(Box::new(rlt::Application { expr, params })),
        })
        .parse(input)
}

fn top_expr(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    alt((
        match_token(Sub).map(|it| UnaryOperationSymbol::Neg(Symbol::from_located(it))),
        match_token(NotLogic).map(|it| UnaryOperationSymbol::Not(Symbol::from_located(it))),
        match_token(NotBit).map(|it| UnaryOperationSymbol::Inv(Symbol::from_located(it))),
        match_token(Plus).map(|it| UnaryOperationSymbol::Plus(Symbol::from_located(it))),
    ))
    .and(top_expr)
    .map(|it| rlt::Operation::TopUnary {
        operator: it.0,
        expr: Box::new(it.1),
    })
    .or(application)
    .context(function!())
    .parse(input)
}

fn pow_expr(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    right_fold(
        top_expr.and(match_token(Pow).and(pow_expr).opt()),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Pow(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn mul_expr(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        pow_expr.and(many0(
            alt((match_token(Times), match_token(Div), match_token(Mod))).and(pow_expr),
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

fn add_expr(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        mul_expr.and(many0(
            alt((match_token(Plus), match_token(Sub))).and(mul_expr),
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

fn complex_cmp(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        add_expr.and(many0(match_token(Spaceship).and(add_expr))),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::ComplexComparison(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

fn compound_cmp(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        complex_cmp.and(many0(
            alt((
                match_token(LessEquals),
                match_token(NotEquiv),
                match_token(Equiv),
                match_token(GreaterEquals),
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

fn simple_cmp(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        compound_cmp.and(many0(
            alt((match_token(Less), match_token(Greater))).and(compound_cmp),
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

fn bit_expr(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        simple_cmp.and(many0(
            alt((match_token(OrBit), match_token(AndBit), match_token(XorBit))).and(simple_cmp),
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

fn logic_expr(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    left_fold(
        bit_expr.and(many0(
            alt((match_token(OrLogic), match_token(AndLogic))).and(bit_expr),
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

fn assign_expr(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    right_fold(
        logic_expr.and(match_token(Equals).and(assign_expr).opt()),
        |a, op, b| rlt::Operation::Binary {
            left: Box::new(a),
            operation: BinaryOperationSymbol::Assign(op),
            right: Box::new(b),
        },
    )
    .context(function!())
    .parse(input)
}

#[inline]
pub(super) fn grammar(input: PackedTokenStream) -> ParseResult<rlt::Operation> {
    assign_expr(input)
}
