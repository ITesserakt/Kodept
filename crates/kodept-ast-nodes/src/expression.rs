use crate::block_level::InitVar;
use crate::code_flow::IfExpr;
use crate::consts::Const;
use crate::literal::{Literal, Tuple};
use crate::properties::{Lhs, Rhs};
use crate::term::{Ref, ReferenceContext};
use crate::types::{NonTyParam, Ty, TyParam};
use crate::Dispatcher;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::relationship::Relationship;
use kodept_ast::experimental::{AstBuilder, Dispatch, DispatchContext, FromSyntax, SpawnContext};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::SpawnedIn;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_rlt::prelude::{Application, Expression, ExpressionBlock, Operation};
use std::convert::Infallible;

#[derive(Debug, PartialEq, Component)]
pub struct Exprs;

#[derive(Debug, PartialEq, Component)]
pub struct App;

#[derive(Debug, PartialEq, Component)]
pub struct Lambda;

#[derive(Debug, PartialEq, Component)]
pub enum BinExpr {
    Access,
    Assign,
}

derive_node!(Exprs);
relation!(Exprs => children InitVar);
relation!(Exprs => children Exprs);
relation!(Exprs => children App);
relation!(Exprs => children Lambda);
relation!(Exprs => children IfExpr);
relation!(Exprs => children BinExpr);
relation!(Exprs => children Ref);
relation!(Exprs => children Ty);
relation!(Exprs => children Literal);
relation!(Exprs => children Tuple);
relation!(Exprs => children Const);

derive_node!(App);
relation!(App => or Lhs(optional Exprs));
relation!(App => or Lhs(optional App));
relation!(App => or Lhs(optional Lambda));
relation!(App => or Lhs(optional IfExpr));
relation!(App => or Lhs(optional BinExpr));
relation!(App => or Lhs(optional Ref));
relation!(App => or Lhs(optional Ty));
relation!(App => or Lhs(optional Literal));
relation!(App => or Lhs(optional Tuple));
relation!(App => or Rhs(children Exprs));
relation!(App => or Rhs(children App));
relation!(App => or Rhs(children Lambda));
relation!(App => or Rhs(children IfExpr));
relation!(App => or Rhs(children BinExpr));
relation!(App => or Rhs(children Ref));
relation!(App => or Rhs(children Ty));
relation!(App => or Rhs(children Literal));
relation!(App => or Rhs(children Tuple));

derive_node!(Lambda);
relation!(Lambda => children TyParam);
relation!(Lambda => children NonTyParam);
relation!(Lambda => child Exprs);

derive_node!(BinExpr);
relation!(BinExpr => or Lhs(optional Exprs));
relation!(BinExpr => or Lhs(optional App));
relation!(BinExpr => or Lhs(optional Lambda));
relation!(BinExpr => or Lhs(optional Ref));
relation!(BinExpr => or Lhs(optional Ty));
relation!(BinExpr => or Lhs(optional BinExpr));
relation!(BinExpr => or Lhs(optional Tuple));
relation!(BinExpr => or Lhs(optional Literal));
relation!(BinExpr => or Lhs(optional IfExpr));

relation!(BinExpr => or Rhs(optional Exprs));
relation!(BinExpr => or Rhs(optional App));
relation!(BinExpr => or Rhs(optional Lambda));
relation!(BinExpr => or Rhs(optional Ref));
relation!(BinExpr => or Rhs(optional Ty));
relation!(BinExpr => or Rhs(optional BinExpr));
relation!(BinExpr => or Rhs(optional Tuple));
relation!(BinExpr => or Rhs(optional Literal));
relation!(BinExpr => or Rhs(optional IfExpr));

impl FromSyntax<ExpressionBlock> for Exprs {
    type Error = crate::Error;

    fn from_syntax<R: Relationship>(
        node: &ExpressionBlock,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        Ok(AstBuilder::new(Exprs)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner)
            .with_dispatches::<Dispatcher<_>, _, _>(node.expression.as_ref(), source)?
            .finish())
    }
}

impl FromSyntax<Application> for App {
    type Error = crate::Error;

    fn from_syntax<R: Relationship>(
        node: &Application,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = AstBuilder::new(App)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);
        builder.with_dispatch::<Dispatcher<_>, Lhs, _>(&node.expr, source)?;
        if let Some(params) = &node.params {
            builder.with_dispatches::<Dispatcher<_>, Rhs, _>(params.inner.as_ref(), source)?;
        }

        Ok(builder.finish())
    }
}

impl FromSyntax<kodept_rlt::prelude::Lambda> for Lambda {
    type Error = crate::Error;

    fn from_syntax<R: Relationship>(
        node: &kodept_rlt::prelude::Lambda,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = AstBuilder::new(Lambda)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);
        builder.with_dispatch_fn(node.expr.as_ref(), |it, spawner| {
            AstBuilder::new(Exprs)
                .with_property(SourceSpan(it.bounds()))
                .spawn_in((spawner, it))
                .with_dispatch::<Dispatcher<_>, _, _>(it, source)
                .map(|it| it.finish_any())
        })?;
        builder.with_dispatches::<Dispatcher<_>, _, _>(node.binds.inner.as_ref(), source)?;
        Ok(builder.finish())
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Operation>
where
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
    type Node = Operation;
    type Error = crate::Error;

    fn dispatch(
        self,
        mut spawner: DispatchContext<R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            Operation::Block(x) => spawner.forward::<_, Exprs>(x, source),
            Operation::Access { left, right, .. } => Ok(AstBuilder::new(BinExpr::Access)
                .with_property(SourceSpan(left.bounds() + right.bounds()))
                .spawn_in((spawner, self.0))
                .with_dispatch::<Dispatcher<_>, Lhs, _>(left.as_ref(), source)?
                .with_dispatch::<Dispatcher<_>, Rhs, _>(left.as_ref(), source)?
                .finish_any()),
            Operation::Unary { operator, expr } => {
                let context = ReferenceContext::global(["Core", "Traits"]);
                let ident = match operator {
                    UnaryOperationSymbol::Neg(_) => "neg".into(),
                    UnaryOperationSymbol::Not(_) => "not".into(),
                    UnaryOperationSymbol::Inv(_) => "inv".into(),
                    UnaryOperationSymbol::Plus(_) => "pos".into(),
                };

                Ok(AstBuilder::new(App)
                    .with_property(SourceSpan(self.0.bounds()))
                    .spawn_in((spawner, self.0))
                    .with_dispatch_fn(operator, |it, spawner: DispatchContext<_, Lhs, _>| {
                        Ok::<_, Infallible>(
                            AstBuilder::new(Ref { context, ident })
                                .with_property(SourceSpan(it.bounds()))
                                .spawn_in((spawner, it))
                                .finish_any(),
                        )
                    })?
                    .with_dispatch::<Dispatcher<_>, Rhs, _>(expr.as_ref(), source)?
                    .finish_any())
            }
            Operation::Binary {
                left,
                operation,
                right,
            } => {
                let context = ReferenceContext::global(["Core", "Traits"]);
                let ident = match operation {
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
                        return Ok(AstBuilder::new(BinExpr::Assign)
                            .with_property(SourceSpan(self.0.bounds()))
                            .spawn_in((spawner, self.0))
                            .with_dispatch::<Dispatcher<_>, Lhs, _>(left.as_ref(), source)?
                            .with_dispatch::<Dispatcher<_>, Rhs, _>(right.as_ref(), source)?
                            .finish_any());
                    }
                };

                Ok(AstBuilder::new(App)
                    .with_property(SourceSpan(self.0.bounds()))
                    .spawn_in((spawner, self.0))
                    .with_dispatch_fn(operation, |it, spawner: DispatchContext<_, Lhs, _>| {
                        Ok::<_, Infallible>(
                            AstBuilder::new(Ref { context, ident })
                                .with_property(SourceSpan(it.bounds()))
                                .spawn_in((spawner, it))
                                .finish_any(),
                        )
                    })?
                    .with_dispatch::<Dispatcher<_>, Rhs, _>(left.as_ref(), source)?
                    .with_dispatch::<Dispatcher<_>, Rhs, _>(right.as_ref(), source)?
                    .finish_any())
            }
            Operation::Application(x) => spawner.forward::<_, App>(x, source),
            Operation::Expression(x) => spawner.dispatch::<Dispatcher<_>>(x, source),
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Expression>
where
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
{
    type Node = Expression;
    type Error = crate::Error;

    fn dispatch(
        self,
        mut spawner: DispatchContext<R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            Expression::Lambda(x) => spawner.forward::<_, Lambda>(x, source),
            Expression::Term(x) => Ok(spawner.dispatch::<Dispatcher<_>>(x, source)?),
            Expression::Literal(x) => spawner.dispatch::<Dispatcher<_>>(x, source),
            Expression::If(x) => spawner.forward::<_, IfExpr>(x, source),
        }
    }
}
