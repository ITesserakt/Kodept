use crate::expression::{App, BinExpr, Exprs, Lambda};
use crate::literal::{Literal, Tuple};
use crate::properties::Condition;
use crate::term::Ref;
use crate::types::Ty;
use crate::utils::{unwrap_body, unwrap_operation};
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;

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
relation!(IfExpr => or Condition(children Ref));
relation!(IfExpr => or Condition(children Ty));
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
relation!(ElifExpr => or Condition(children Ref));
relation!(ElifExpr => or Condition(children Ty));
relation!(ElifExpr => or Condition(children Literal));
relation!(ElifExpr => or Condition(children Tuple));
relation!(ElifExpr => child Exprs);

derive_node!(ElseExpr);
relation!(ElseExpr => child Exprs);

impl FromSyntax<kodept_rlt::prelude::IfExpr> for IfExpr {
    type Bundle = impl Bundle;

    fn from_syntax(node: &kodept_rlt::prelude::IfExpr, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(IfExpr)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(&node.condition, source, unwrap_operation)
            .with_children::<_, ElifExpr, _>(node.elif.as_ref(), source)
            .with_opt_child::<_, ElseExpr, _>(node.el.as_ref(), source)
            .with_dyn_child(&node.body, source, unwrap_body::<_, (), _>)
            .build()
    }
}

impl FromSyntax<kodept_rlt::prelude::ElifExpr> for ElifExpr {
    type Bundle = impl Bundle;

    fn from_syntax(node: &kodept_rlt::prelude::ElifExpr, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(ElifExpr)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(&node.condition, source, unwrap_operation)
            .with_dyn_child(&node.body, source, unwrap_body::<_, (), _>)
            .build()
    }
}

impl FromSyntax<kodept_rlt::prelude::ElseExpr> for ElseExpr {
    type Bundle = impl Bundle;

    fn from_syntax(node: &kodept_rlt::prelude::ElseExpr, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(ElseExpr)
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(&node.body, source, unwrap_body)
            .build()
    }
}
