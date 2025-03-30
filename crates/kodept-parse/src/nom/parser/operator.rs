use nom::branch::alt;
use nom::combinator::opt;
use nom::error::context;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::Parser;
use nonempty_collections::NEVec;

use crate::lexer::PackedToken::*;
use crate::nom::parser::macros::function;
use crate::nom::parser::utils::{comma_separated0, match_token, paren_enclosed};
use crate::nom::parser::{expression, PParser, PResult};
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use kodept_rlt::prelude as rlt;
use kodept_rlt::new_types::{
    BinaryOperationSymbol, Enclosed, Symbol, UnaryOperationSymbol,
};

fn left_fold<I, T, P, R>(
    parser: P,
    produce: impl Fn(R, Symbol, T) -> R,
) -> impl Parser<I, Output = R, Error = P::Error>
where
    P: Parser<I, Output = (T, Vec<(PackedTokenMatch, T)>)>,
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

fn right_fold<I, T, P, R>(
    parser: P,
    produce: impl Fn(R, Symbol, T) -> R,
) -> impl Parser<I, Output = R, Error = P::Error>
where
    P: Parser<I, Output = (T, Option<(PackedTokenMatch, T)>)>,
    R: From<T>,
{
    parser.map(move |(a, tail)| match tail {
        None => a.into(),
        Some((op, b)) => produce(a.into(), Symbol::from_located(op), b),
    })
}

fn atom<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        alt((
            delimited(match_token(LParen), grammar(), match_token(RParen)),
            expression::grammar().map(rlt::Operation::Expression),
        )),
    )
}

fn access<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold((atom(), many0((match_token(Dot), atom()))), |a, op, b| {
            rlt::Operation::Access {
                left: Box::new(a),
                dot: op,
                right: Box::new(b),
            }
        }),
    )
}

fn parameters<'t>() -> impl PParser<'t, Enclosed<Box<[rlt::Operation]>>> {
    context(function!(), paren_enclosed(comma_separated0(grammar()))).map(|it| it.into())
}

fn application<'t>() -> impl PParser<'t, rlt::Operation> {
    context(function!(), (access(), opt(parameters()))).map(|(expr, params)| match params {
        None => expr,
        Some(_) => rlt::Operation::Application(Box::new(rlt::Application { expr, params })),
    })
}

fn top_expr(input: PackedTokenStream) -> PResult<rlt::Operation> {
    context(
        function!(),
        alt((
            match_token(Sub).map(|it| UnaryOperationSymbol::Neg(Symbol::from_located(it))),
            match_token(NotLogic).map(|it| UnaryOperationSymbol::Not(Symbol::from_located(it))),
            match_token(NotBit).map(|it| UnaryOperationSymbol::Inv(Symbol::from_located(it))),
            match_token(Plus).map(|it| UnaryOperationSymbol::Plus(Symbol::from_located(it))),
        ))
        .and(top_expr),
    )
    .map(|it| rlt::Operation::TopUnary {
        operator: it.0,
        expr: Box::new(it.1),
    })
    .or(application())
    .parse(input)
}

fn pow_expr(input: PackedTokenStream) -> PResult<rlt::Operation> {
    context(
        function!(),
        right_fold(
            top_expr.and(opt(match_token(Pow).and(pow_expr))),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::Pow(op),
                right: Box::new(b),
            },
        ),
    )
    .parse(input)
}

fn mul_expr<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold(
            pow_expr.and(many0(
                alt((match_token(Times), match_token(Div), match_token(Mod))).and(pow_expr),
            )),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::Mul(op),
                right: Box::new(b),
            },
        ),
    )
}

fn add_expr<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold(
            mul_expr().and(many0(
                alt((match_token(Plus), match_token(Sub))).and(mul_expr()),
            )),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::Add(op),
                right: Box::new(b),
            },
        ),
    )
}

fn complex_cmp<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold(
            add_expr().and(many0(match_token(Spaceship).and(add_expr()))),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::ComplexComparison(op),
                right: Box::new(b),
            },
        ),
    )
}

fn compound_cmp<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold(
            complex_cmp().and(many0(
                alt((
                    match_token(LessEquals),
                    match_token(NotEquiv),
                    match_token(Equiv),
                    match_token(GreaterEquals),
                ))
                .and(complex_cmp()),
            )),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::CompoundComparison(op),
                right: Box::new(b),
            },
        ),
    )
}

fn simple_cmp<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold(
            compound_cmp().and(many0(
                alt((match_token(Less), match_token(Greater))).and(compound_cmp()),
            )),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::Comparison(op),
                right: Box::new(b),
            },
        ),
    )
}

fn bit_expr<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold(
            simple_cmp().and(many0(
                alt((match_token(OrBit), match_token(AndBit), match_token(XorBit)))
                    .and(simple_cmp()),
            )),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::Bit(op),
                right: Box::new(b),
            },
        ),
    )
}

fn logic_expr<'t>() -> impl PParser<'t, rlt::Operation> {
    context(
        function!(),
        left_fold(
            bit_expr().and(many0(
                alt((match_token(OrLogic), match_token(AndLogic))).and(bit_expr()),
            )),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::Logic(op),
                right: Box::new(b),
            },
        ),
    )
}

fn assign_expr(input: PackedTokenStream) -> PResult<rlt::Operation> {
    context(
        function!(),
        right_fold(
            logic_expr().and(opt(match_token(Equals).and(assign_expr))),
            |a, op, b| rlt::Operation::Binary {
                left: Box::new(a),
                operation: BinaryOperationSymbol::Assign(op),
                right: Box::new(b),
            },
        ),
    )
    .parse(input)
}

#[inline]
pub(super) fn grammar<'t>() -> impl PParser<'t, rlt::Operation> {
    assign_expr
}
