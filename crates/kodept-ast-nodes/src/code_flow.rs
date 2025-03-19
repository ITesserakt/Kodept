use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::literal::{Literal, Tuple};
use crate::term::Ref;
use crate::utils::unwrap_body;
use crate::Unit;
use kodept_ast::{derive_node, relation};
use kodept_ast::external::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use crate::properties::Condition;

#[derive(Debug, PartialEq, Component)]
pub struct IfExpr;

#[derive(Debug, PartialEq, Component)]
pub struct ElifExpr;

#[derive(Debug, PartialEq, Component)]
pub struct ElseExpr;

derive_node!(IfExpr);

relation!(IfExpr => or Condition(children Exprs));
relation!(IfExpr => or Condition(children App));
relation!(IfExpr => or Condition(children Lambda));
relation!(IfExpr => or Condition(children IfExpr));
relation!(IfExpr => or Condition(children BinExpr));
relation!(IfExpr => or Condition(children UnExpr));
relation!(IfExpr => or Condition(children Ref));
relation!(IfExpr => or Condition(children Literal));
relation!(IfExpr => or Condition(children Tuple));
relation!(IfExpr => child Exprs);
relation!(IfExpr => children ElifExpr);
relation!(IfExpr => optional ElseExpr);

derive_node!(ElifExpr);
relation!(ElifExpr => or Condition(children Exprs));
relation!(ElifExpr => or Condition(children App));
relation!(ElifExpr => or Condition(children Lambda));
relation!(ElifExpr => or Condition(children IfExpr));
relation!(ElifExpr => or Condition(children BinExpr));
relation!(ElifExpr => or Condition(children UnExpr));
relation!(ElifExpr => or Condition(children Ref));
relation!(ElifExpr => or Condition(children Literal));
relation!(ElifExpr => or Condition(children Tuple));
relation!(ElifExpr => child Exprs);

derive_node!(ElseExpr);
relation!(ElseExpr => child Exprs);

impl FromSyntax for IfExpr {
    type Syntax = kodept_rlt::prelude::IfExpr;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, IfExpr).with_children(source, pool, |scope| {
            scope.choose(Unit, [&node.condition]);
            scope.many::<ElifExpr, _>(node.elif.as_ref());
            scope.many::<ElseExpr, _>(node.el.as_ref());
            unwrap_body::<_, _, ()>(&node.body, scope);
        })
    }
}

impl FromSyntax for ElifExpr {
    type Syntax = kodept_rlt::prelude::ElifExpr;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, ElifExpr).with_children(source, pool, |scope| {
            scope.choose(Unit, [&node.condition]);
            unwrap_body::<_, _, ()>(&node.body, scope);
        })
    }
}

impl FromSyntax for ElseExpr {
    type Syntax = kodept_rlt::prelude::ElseExpr;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, ElseExpr).with_children(source, pool, |scope| {
            unwrap_body(&node.body, scope);
        })
    }
}
