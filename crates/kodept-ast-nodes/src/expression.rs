use crate::block_level::InitVar;
use crate::code_flow::IfExpr;
use crate::consts::Const;
use crate::literal::{Literal, Tuple};
use crate::properties::{Lhs, Rhs};
use crate::term::Ref;
use crate::types::{NonTyParam, Ty, TyParam};
use crate::utils::{unwrap_block_level, unwrap_operation, unwrap_parameter};
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude::{Application, ExpressionBlock};
use std::convert::identity;

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
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(
        node: &ExpressionBlock,
        source: impl CodeHolder,
    ) -> Result<Self::Bundle, Self::Error> {
        Ok(ASTBuilder::new(Exprs)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_children(node.expression.as_ref(), |it, spawner| {
                unwrap_block_level(it, spawner, source)
            })?
            .build())
    }
}

impl FromSyntax<Application> for App {
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(
        node: &Application,
        source: impl CodeHolder,
    ) -> Result<Self::Bundle, Self::Error> {
        Ok(ASTBuilder::new(App)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(&node.expr, source, unwrap_operation::<_, Lhs, _>)?
            .with_opt_dyn_children(
                node.params.as_ref().map(|it| it.inner.as_ref()),
                |it, spawner| unwrap_operation::<_, Rhs, _>(it, spawner, source),
            )?
            .build())
    }
}

impl FromSyntax<kodept_rlt::prelude::Lambda> for Lambda {
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(
        node: &kodept_rlt::prelude::Lambda,
        source: impl CodeHolder,
    ) -> Result<Self::Bundle, Self::Error> {
        Ok(ASTBuilder::new(Lambda)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(node.expr.as_ref(), source, |it, spawner, source| {
                Ok(spawner.spawn_raw(
                    ASTBuilder::new(Exprs)
                        .with_property(SourceSpan(it.bounds()))
                        .with_dyn_child(it, source, unwrap_operation)?,
                    it,
                    identity,
                ))
            })?
            .with_dyn_children(node.binds.inner.as_ref(), |it, spawner| {
                unwrap_parameter(it, spawner, source)
            })?
            .build())
    }
}
