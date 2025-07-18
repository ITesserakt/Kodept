use std::convert::identity;

use crate::block_level::InitVar;
use crate::code_flow::IfExpr;
use crate::consts::Const;
use crate::expression::{App, BinExpr, Exprs, Lambda};
use crate::literal::{Literal, Tuple};
use crate::properties::{Lhs, Rhs};
use crate::term::{Ref, ReferenceContext};
use crate::types::{NonTyParam, ProdTy, Ty, TyParam};
use crate::Either;
use kodept_ast::arity::Arity;
use kodept_ast::prelude::CodeHolder;
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, BundleUnion, NodeSpawner};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_rlt::prelude::{self as rlt};
use kodept_rlt::prelude::{BlockLevelNode, Body, Expression, Operation, Parameter, Type};

pub(crate) fn unwrap_type<R, T, A>(
    node: &Type,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<ProdTy, T, Arity = A>,
{
    Ok(match node {
        Type::ContextualReference(context, ident) => {
            let (is_global, context_items) = context.unfold();
            let ident = source.get_chunk_located(ident);
            let context_items = context_items
                .into_iter()
                .map(|it| source.get_chunk_located(it));
            let context = if is_global.is_some() {
                ReferenceContext::global(context_items)
            } else {
                ReferenceContext::local(context_items)
            };
            spawner.spawn_raw(
                ASTBuilder::new(Ty { context, ident }).with_property(SourceSpan(node.bounds())),
                node,
                Either::v31,
            )
        }
        Type::Reference(x) => spawner.spawn::<_, Ty, _>(x, source, Either::v32)?,
        Type::Tuple(x) => spawner.spawn::<_, ProdTy, _>(x, source, Either::v33)?,
    })
}

pub(crate) fn unwrap_body<R, T, A>(
    node: &Body,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<Exprs, T, Arity = A>,
{
    Ok(match node {
        Body::Block(x) => spawner.spawn::<_, Exprs, _>(x, source, Either::v21)?,
        Body::Simplified { expression, .. } => spawner.spawn_raw(
            ASTBuilder::new(Exprs)
                .with_property(SourceSpan(expression.bounds()))
                .with_dyn_child(expression, source, unwrap_block_level)?,
            expression,
            Either::v22,
        ),
    })
}

pub(crate) fn unwrap_block_level<R, T, A>(
    node: &BlockLevelNode,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<InitVar, T, Arity = A>,
    R: HasChild<Exprs, T, Arity = A>,
    R: HasChild<Const, T, Arity = A>,
    R: HasChild<App, T, Arity = A>,
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<BinExpr, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
{
    Ok(match node {
        BlockLevelNode::InitVar(x) => spawner.spawn::<_, InitVar, _>(x, source, Either::v41)?,
        BlockLevelNode::Block(x) => spawner.spawn::<_, Exprs, _>(x, source, Either::v42)?,
        BlockLevelNode::Function(x) => spawner.spawn::<_, Const, _>(x, source, Either::v43)?,
        BlockLevelNode::Operation(x) => Either::v44(unwrap_operation(x, spawner, source)?),
    })
}

pub(crate) fn unwrap_parameter<R, T, A>(
    node: &Parameter,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<TyParam, T, Arity = A>,
    R: HasChild<NonTyParam, T, Arity = A>,
{
    match &node {
        Parameter::Typed(x) => spawner.spawn::<_, TyParam, _>(x, source, Either::v21),
        Parameter::Untyped(x) => spawner.spawn::<_, NonTyParam, _>(x, source, Either::v22),
    }
}

pub(crate) fn unwrap_operation<R, T, A>(
    node: &Operation,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<Exprs, T, Arity = A>,
    R: HasChild<BinExpr, T, Arity = A>,
    R: HasChild<App, T, Arity = A>,
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
{
    match node {
        Operation::Block(x) => spawner.spawn::<_, Exprs, _>(x, source, Either::v71),
        Operation::Access { left, right, .. } => {
            let builder = ASTBuilder::new(BinExpr::Access)
                .with_property(SourceSpan(left.bounds() + right.bounds()))
                .with_dyn_child(left.as_ref(), source, unwrap_operation::<_, Lhs, _>)?
                .with_dyn_child(right.as_ref(), source, unwrap_operation::<_, Rhs, _>)?;

            Ok(spawner.spawn_raw(builder, node, Either::v72))
        }
        Operation::Binary {
            left,
            operation,
            right,
        } => {
            let context = ReferenceContext::global(["Core", "Traits"]);

            let build_ref_to = |symbol: &BinaryOperationSymbol, ident| {
                let ref_builder = ASTBuilder::new(Ref { context, ident })
                    .with_property(SourceSpan(symbol.bounds()));
                let app_builder = ASTBuilder::new(App)
                    .with_dyn_child(
                        symbol,
                        source,
                        move |x, spawner: &mut NodeSpawner<_, Lhs, _>, _| {
                            Ok(spawner.spawn_raw(ref_builder, x, identity))
                        },
                    )?
                    .with_dyn_children([left.as_ref(), right.as_ref()], |x, spawner| {
                        unwrap_operation::<_, Rhs, _>(x, spawner, source)
                    })?
                    .with_property(SourceSpan(left.bounds() + right.bounds()));

                Ok(app_builder)
            };

            let value = match operation {
                BinaryOperationSymbol::Pow(_) => "pow".into(),
                BinaryOperationSymbol::Mul(_) => "mul".into(),
                BinaryOperationSymbol::Div(_) => "div".into(),
                BinaryOperationSymbol::Rem(_) => "rem".into(),
                BinaryOperationSymbol::Add(_) => "add".into(),
                BinaryOperationSymbol::Sub(_) => "sub".into(),
                BinaryOperationSymbol::ComplexComparison(_) => "spaceship".into(),
                BinaryOperationSymbol::LessEq(_) => "leq".into(),
                BinaryOperationSymbol::NEq(_) => "neq".into(),
                BinaryOperationSymbol::Eq(_) => "eq".into(),
                BinaryOperationSymbol::GreaterEq(_) => "geq".into(),
                BinaryOperationSymbol::Less(_) => "less".into(),
                BinaryOperationSymbol::Greater(_) => "greater".into(),
                BinaryOperationSymbol::Or(_) => "or".into(),
                BinaryOperationSymbol::And(_) => "and".into(),
                BinaryOperationSymbol::Xor(_) => "xor".into(),
                BinaryOperationSymbol::Disjunction(_) => "disj".into(),
                BinaryOperationSymbol::Conjunction(_) => "conj".into(),
                BinaryOperationSymbol::Assign(_) => {
                    let builder = ASTBuilder::new(BinExpr::Assign)
                        .with_dyn_child(left.as_ref(), source, unwrap_operation::<_, Lhs, _>)?
                        .with_dyn_child(right.as_ref(), source, unwrap_operation::<_, Rhs, _>)?
                        .with_property(SourceSpan(node.bounds()));

                    return Ok(spawner.spawn_raw(builder, node, Either::v74));
                }
            };
            Ok(spawner.spawn_raw(build_ref_to(operation, value)?, node, Either::v73))
        }
        Operation::Unary { operator, expr } => {
            let context = ReferenceContext::global(["Core", "Traits"]);

            let build_ref_to = |symbol: &UnaryOperationSymbol, ident| {
                let ref_builder = ASTBuilder::new(Ref { context, ident })
                    .with_property(SourceSpan(symbol.bounds()));
                let app_builder = ASTBuilder::new(App)
                    .with_dyn_child(
                        symbol,
                        source,
                        move |x, spawner: &mut NodeSpawner<_, Lhs, _>, _| {
                            Ok(spawner.spawn_raw(ref_builder, x, identity))
                        },
                    )?
                    .with_dyn_child(expr.as_ref(), source, unwrap_operation::<_, Rhs, _>)?
                    .with_property(SourceSpan(expr.bounds()));

                Ok(app_builder)
            };

            let value = match operator {
                UnaryOperationSymbol::Neg(_) => "neg".into(),
                UnaryOperationSymbol::Not(_) => "not".into(),
                UnaryOperationSymbol::Inv(_) => "inv".into(),
                UnaryOperationSymbol::Plus(_) => "pos".into(),
            };
            Ok(spawner.spawn_raw(build_ref_to(operator, value)?, node, Either::v75))
        }
        Operation::Application(x) => spawner.spawn::<_, App, _>(x, source, Either::v76),
        Operation::Expression(x) => unwrap_expression(x, spawner, source).map(Either::v77),
    }
}

pub(crate) fn unwrap_expression<R, T, A>(
    node: &Expression,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
{
    Ok(match node {
        Expression::Lambda(x) => spawner.spawn::<_, Lambda, _>(x, source, Either::v41)?,
        Expression::Term(x) => Either::v42(unwrap_term(x, spawner, source)?),
        Expression::Literal(x) => Either::v43(unwrap_literal(x, spawner, source)?),
        Expression::If(x) => spawner.spawn::<_, IfExpr, _>(x, source, Either::v44)?,
    })
}

fn unwrap_term<R, T, A>(
    node: &rlt::Term,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Ty, T, Arity = A>,
{
    Ok(match node {
        rlt::Term::Reference(x) => {
            let ident = source.get_chunk_located(x);
            let value = Ref {
                context: ReferenceContext::empty(false),
                ident,
            };
            spawner.spawn_raw(
                ASTBuilder::new(value).with_property(SourceSpan(x.bounds())),
                node,
                Either::v41,
            )
        }
        rlt::Term::ContextualReference(x) => {
            let ident = source.get_chunk_located(&x.inner);
            let (is_global, items) = x.context.unfold();
            let context = if is_global.is_some() {
                ReferenceContext::global(items.into_iter().map(|it| source.get_chunk_located(it)))
            } else {
                ReferenceContext::local(items.into_iter().map(|it| source.get_chunk_located(it)))
            };
            let value = Ref { context, ident };
            spawner.spawn_raw(
                ASTBuilder::new(value).with_property(SourceSpan(x.bounds())),
                node,
                Either::v42,
            )
        }
        rlt::Term::Constant(x) => {
            let ident = source.get_chunk_located(x);
            let value = Ty {
                context: ReferenceContext::empty(false),
                ident,
            };
            spawner.spawn_raw(
                ASTBuilder::new(value).with_property(SourceSpan(x.bounds())),
                node,
                Either::v43,
            )
        }
        rlt::Term::ContextualConstant(x) => {
            let ident = source.get_chunk_located(&x.inner);
            let (is_global, items) = x.context.unfold();
            let context = if is_global.is_some() {
                ReferenceContext::global(items.into_iter().map(|it| source.get_chunk_located(it)))
            } else {
                ReferenceContext::local(items.into_iter().map(|it| source.get_chunk_located(it)))
            };
            let value = Ty { context, ident };
            spawner.spawn_raw(
                ASTBuilder::new(value).with_property(SourceSpan(x.bounds())),
                node,
                Either::v44,
            )
        }
    })
}

pub(crate) fn unwrap_literal<R, T, A>(
    node: &rlt::Literal,
    spawner: &mut NodeSpawner<R, T, A>,
    source: impl CodeHolder,
) -> Result<impl BundleUnion, crate::Error>
where
    A: Arity,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
{
    Ok(match node {
        rlt::Literal::Tuple(x) => spawner.spawn_raw(
            ASTBuilder::new(Tuple)
                .with_property(SourceSpan(x.left.0 + x.right.0))
                .with_dyn_children(x.inner.as_ref(), |it, spawner| {
                    unwrap_operation(it, spawner, source)
                })?,
            node,
            Either::v21,
        ),
        _ => {
            let text = source.get_chunk_located(node);
            let value = Literal::from_str(node, text)?;
            spawner.spawn_raw(
                ASTBuilder::new(value).with_property(SourceSpan(node.bounds())),
                node,
                Either::v22,
            )
        }
    })
}
