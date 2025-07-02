use crate::block_level::InitVar;
use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::function::FuncDecl;
use crate::literal::{Literal, Tuple};
use crate::properties::{Lhs, Rhs};
use crate::term::Ref;
use crate::types::{NonTyParam, ProdTy, Ty, TyParam};
use crate::Either;
use kodept_ast::arity::Arity;
use kodept_ast::prelude::CodeHolder;
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, BundleUnion, NodeSpawner};
use kodept_rlt::exported::{Located, SpanBounds};
use kodept_rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_rlt::prelude::{self as rlt};
use kodept_rlt::prelude::{BlockLevelNode, Body, Expression, Operation, Parameter, Type};

pub(crate) fn unwrap_type<R, T, A>(
    node: &Type,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> impl BundleUnion
where
    A: Arity,
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<ProdTy, T, Arity = A>,
{
    match node {
        Type::Reference(x) => spawner.spawn::<_, Ty, _>(x, source, Either::Left),
        Type::Tuple(x) => spawner.spawn::<_, ProdTy, _>(x, source, Either::Right),
    }
}

pub(crate) fn unwrap_body<R, T, A>(
    node: &Body,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> impl BundleUnion
where
    A: Arity,
    R: HasChild<Exprs, T, Arity = A>,
{
    match node {
        Body::Block(x) => spawner.spawn::<_, Exprs, _>(x, source, Either::Left),
        Body::Simplified { expression, .. } => spawner.spawn_raw(
            ASTBuilder::new(Exprs)
                .with_property(SourceSpan(expression.bounds()))
                .with_dyn_child(expression, source, unwrap_block_level),
            expression,
            Either::Right,
        ),
    }
}

pub(crate) fn unwrap_block_level<R, T, A>(
    node: &BlockLevelNode,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> impl BundleUnion
where
    A: Arity,
    R: HasChild<InitVar, T, Arity = A>,
    R: HasChild<Exprs, T, Arity = A>,
    R: HasChild<FuncDecl, T, Arity = A>,
    R: HasChild<BinExpr, T, Arity = A>,
    R: HasChild<UnExpr, T, Arity = A>,
    R: HasChild<App, T, Arity = A>,
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
{
    match node {
        BlockLevelNode::InitVar(x) => {
            spawner.spawn::<_, InitVar, _>(x, source, |x| Either::Left(Either::Left(x)))
        }
        BlockLevelNode::Block(x) => {
            spawner.spawn::<_, Exprs, _>(x, source, |x| Either::Left(Either::Right(x)))
        }
        BlockLevelNode::Function(x) => {
            spawner.spawn::<_, FuncDecl, _>(x, source, |x| Either::Right(Either::Left(x)))
        }
        BlockLevelNode::Operation(x) => {
            Either::Right(Either::Right(unwrap_operation(x, spawner, source)))
        }
    }
}

pub(crate) fn unwrap_parameter<R, T, A>(
    node: &Parameter,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> impl BundleUnion
where
    A: Arity,
    R: HasChild<TyParam, T, Arity = A>,
    R: HasChild<NonTyParam, T, Arity = A>,
{
    match &node {
        Parameter::Typed(x) => spawner.spawn::<_, TyParam, _>(x, source, Either::Left),
        Parameter::Untyped(x) => spawner.spawn::<_, NonTyParam, _>(x, source, Either::Right),
    }
}

pub(crate) fn unwrap_operation<R, T, A>(
    node: &Operation,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> impl BundleUnion
where
    A: Arity,
    R: HasChild<Exprs, T, Arity = A>,
    R: HasChild<BinExpr, T, Arity = A>,
    R: HasChild<UnExpr, T, Arity = A>,
    R: HasChild<App, T, Arity = A>,
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
{
    match node {
        Operation::Block(x) => {
            spawner.spawn::<_, Exprs, _>(x, source, |x| Either::Left(Either::Left(Either::Left(x))))
        }
        Operation::Access { left, right, .. } => spawner.spawn_raw(
            ASTBuilder::new(BinExpr::Access)
                .with_property(SourceSpan(left.bounds() + right.bounds()))
                .with_dyn_child(left.as_ref(), source, unwrap_operation::<_, Lhs, _>)
                .with_dyn_child(right.as_ref(), source, unwrap_operation::<_, Rhs, _>),
            node,
            |x| Either::Left(Either::Left(Either::Right(x))),
        ),
        Operation::Binary {
            left,
            operation,
            right,
        } => {
            let value = match operation {
                BinaryOperationSymbol::Pow(_) => BinExpr::Pow,
                BinaryOperationSymbol::Mul(_) => BinExpr::Mul,
                BinaryOperationSymbol::Div(_) => BinExpr::Div,
                BinaryOperationSymbol::Rem(_) => BinExpr::Mod,
                BinaryOperationSymbol::Add(_) => BinExpr::Add,
                BinaryOperationSymbol::Sub(_) => BinExpr::Sub,
                BinaryOperationSymbol::ComplexComparison(_) => BinExpr::ComplexComparison,
                BinaryOperationSymbol::LessEq(_) => BinExpr::LessEq,
                BinaryOperationSymbol::NEq(_) => BinExpr::NEq,
                BinaryOperationSymbol::Eq(_) => BinExpr::Eq,
                BinaryOperationSymbol::GreaterEq(_) => BinExpr::GreaterEq,
                BinaryOperationSymbol::Less(_) => BinExpr::Less,
                BinaryOperationSymbol::Greater(_) => BinExpr::Greater,
                BinaryOperationSymbol::Or(_) => BinExpr::Or,
                BinaryOperationSymbol::And(_) => BinExpr::And,
                BinaryOperationSymbol::Xor(_) => BinExpr::Xor,
                BinaryOperationSymbol::Disjunction(_) => BinExpr::Disj,
                BinaryOperationSymbol::Conjunction(_) => BinExpr::Conj,
                BinaryOperationSymbol::Assign(_) => BinExpr::Assign,
            };
            spawner.spawn_raw(
                ASTBuilder::new(value)
                    .with_property(SourceSpan(left.bounds() + right.bounds()))
                    .with_dyn_child(left.as_ref(), source, unwrap_operation::<_, Lhs, _>)
                    .with_dyn_child(right.as_ref(), source, unwrap_operation::<_, Rhs, _>),
                node,
                |x| Either::Left(Either::Right(Either::Left(x))),
            )
        }
        Operation::Unary { operator, expr } => {
            let value = match operator {
                UnaryOperationSymbol::Neg(_) => UnExpr::Neg,
                UnaryOperationSymbol::Not(_) => UnExpr::Not,
                UnaryOperationSymbol::Inv(_) => UnExpr::Inv,
                UnaryOperationSymbol::Plus(_) => UnExpr::Plus,
            };
            spawner.spawn_raw(
                ASTBuilder::new(value)
                    .with_property(SourceSpan(operator.location() + expr.bounds()))
                    .with_dyn_child(expr.as_ref(), source, unwrap_operation),
                node,
                |x| Either::Left(Either::Right(Either::Right(x))),
            )
        }
        Operation::Application(x) => {
            spawner.spawn::<_, App, _>(x, source, |x| Either::Right(Either::Left(x)))
        }
        Operation::Expression(x) => {
            Either::Right(Either::Right(unwrap_expression(x, spawner, source)))
        }
    }
}

pub(crate) fn unwrap_expression<R, T, A>(
    node: &Expression,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> impl BundleUnion
where
    A: Arity,
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
{
    match node {
        Expression::Lambda(x) => {
            spawner.spawn::<_, Lambda, _>(x, source, |x| Either::Left(Either::Left(x)))
        }
        Expression::Term(x) => {
            spawner.spawn::<_, Ref, _>(x, source, |x| Either::Left(Either::Right(x)))
        }
        Expression::Literal(x) => Either::Right(Either::Left(unwrap_literal(x, spawner, source))),
        Expression::If(x) => {
            spawner.spawn::<_, IfExpr, _>(x, source, |x| Either::Right(Either::Right(x)))
        }
    }
}

pub(crate) fn unwrap_literal<R, T, A>(
    node: &rlt::Literal,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> impl BundleUnion
where
    A: Arity,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
{
    match node {
        rlt::Literal::Tuple(x) => spawner.spawn_raw(
            ASTBuilder::new(Tuple)
                .with_property(SourceSpan(x.left.0 + x.right.0))
                .with_dyn_children(x.inner.as_ref(), |it, spawner| {
                    unwrap_operation(it, spawner, source)
                }),
            node,
            Either::Left,
        ),
        _ => {
            let text = source.get_chunk_located(node);
            let value = Literal::from_str(node, text);
            spawner.spawn_raw(
                ASTBuilder::new(value).with_property(SourceSpan(node.bounds())),
                node,
                Either::Right,
            )
        }
    }
}
