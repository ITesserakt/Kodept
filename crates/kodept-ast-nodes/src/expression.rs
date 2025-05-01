use crate::block_level::InitVar;
use crate::code_flow::IfExpr;
use crate::constants::Const;
use crate::function::Func;
use crate::literal::{Literal, Tuple};
use crate::properties::{Lhs, Rhs};
use crate::term::Ref;
use crate::types::{NonTyParam, TyParam};
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
    Add,
    Sub,
    Mul,
    Pow,
    Div,
    Mod,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Eq,
    NEq,
    Or,
    And,
    Xor,
    Disj,
    Conj,
    ComplexComparison,
    Assign,
}

#[derive(Debug, PartialEq, Component)]
pub enum UnExpr {
    Neg,
    Not,
    Inv,
    Plus,
}

derive_node!(Exprs);
relation!(Exprs => children InitVar);
relation!(Exprs => children Const);
relation!(Exprs => children Exprs);
relation!(Exprs => children App);
relation!(Exprs => children Lambda);
relation!(Exprs => children IfExpr);
relation!(Exprs => children BinExpr);
relation!(Exprs => children UnExpr);
relation!(Exprs => children Ref);
relation!(Exprs => children Literal);
relation!(Exprs => children Tuple);
relation!(Exprs => children Func);

derive_node!(App);
relation!(App => or Lhs(optional Exprs));
relation!(App => or Lhs(optional App));
relation!(App => or Lhs(optional Lambda));
relation!(App => or Lhs(optional IfExpr));
relation!(App => or Lhs(optional BinExpr));
relation!(App => or Lhs(optional UnExpr));
relation!(App => or Lhs(optional Ref));
relation!(App => or Lhs(optional Literal));
relation!(App => or Lhs(optional Tuple));
relation!(App => or Rhs(optional Exprs));
relation!(App => or Rhs(optional App));
relation!(App => or Rhs(optional Lambda));
relation!(App => or Rhs(optional IfExpr));
relation!(App => or Rhs(optional BinExpr));
relation!(App => or Rhs(optional UnExpr));
relation!(App => or Rhs(optional Ref));
relation!(App => or Rhs(optional Literal));
relation!(App => or Rhs(optional Tuple));

derive_node!(Lambda);
relation!(Lambda => children TyParam);
relation!(Lambda => children NonTyParam);
relation!(Lambda => child Exprs);

derive_node!(BinExpr);
relation!(BinExpr => or Lhs(optional Exprs));
relation!(BinExpr => or Lhs(optional App));
relation!(BinExpr => or Lhs(optional Lambda));
relation!(BinExpr => or Lhs(optional IfExpr));
relation!(BinExpr => or Lhs(optional BinExpr));
relation!(BinExpr => or Lhs(optional UnExpr));
relation!(BinExpr => or Lhs(optional Ref));
relation!(BinExpr => or Lhs(optional Literal));
relation!(BinExpr => or Lhs(optional Tuple));
relation!(BinExpr => or Rhs(children Exprs));
relation!(BinExpr => or Rhs(children App));
relation!(BinExpr => or Rhs(children Lambda));
relation!(BinExpr => or Rhs(children IfExpr));
relation!(BinExpr => or Rhs(children BinExpr));
relation!(BinExpr => or Rhs(children UnExpr));
relation!(BinExpr => or Rhs(children Ref));
relation!(BinExpr => or Rhs(children Literal));
relation!(BinExpr => or Rhs(children Tuple));

derive_node!(UnExpr);
relation!(UnExpr => optional Exprs);
relation!(UnExpr => optional App);
relation!(UnExpr => optional Lambda);
relation!(UnExpr => optional IfExpr);
relation!(UnExpr => optional BinExpr);
relation!(UnExpr => optional UnExpr);
relation!(UnExpr => optional Ref);
relation!(UnExpr => optional Literal);
relation!(UnExpr => optional Tuple);

impl FromSyntax<ExpressionBlock> for Exprs {
    type Bundle = impl Bundle;

    fn from_syntax(node: &ExpressionBlock, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(Exprs)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_children(node.expression.as_ref(), |it, spawner| {
                unwrap_block_level(it, spawner, source)
            })
            .build()
    }
}

impl FromSyntax<Application> for App {
    type Bundle = impl Bundle;

    fn from_syntax(node: &Application, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(App)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(&node.expr, source, unwrap_operation::<_, Lhs, _>)
            .with_opt_dyn_children(
                node.params.as_ref().map(|it| it.inner.as_ref()),
                |it, spawner| unwrap_operation::<_, Rhs, _>(it, spawner, source),
            )
            .build()
    }
}

impl FromSyntax<kodept_rlt::prelude::Lambda> for Lambda {
    type Bundle = impl Bundle;

    fn from_syntax(node: &kodept_rlt::prelude::Lambda, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(Lambda)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(node.expr.as_ref(), source, |it, spawner, source| {
                spawner.spawn_raw(
                    ASTBuilder::new(Exprs)
                        .with_property(SourceSpan(it.bounds()))
                        .with_dyn_child(it, source, unwrap_operation),
                    it,
                    identity,
                )
            })
            .with_dyn_children(node.binds.inner.as_ref(), |it, spawner| {
                unwrap_parameter(it, spawner, source)
            })
            .build()
    }
}
