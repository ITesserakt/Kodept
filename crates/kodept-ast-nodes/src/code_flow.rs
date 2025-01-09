use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::literal::{Literal, Tuple};
use crate::properties::Expr;
use crate::term::Ref;
use crate::utils::unwrap_body;
use crate::Unit;
use kodept_ast::derive_node;
use kodept_ast::external::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::tags::NoTag;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};

#[derive(Debug, PartialEq, Component)]
pub struct IfExpr;

#[derive(Debug, PartialEq, Component)]
pub struct ElifExpr;

#[derive(Debug, PartialEq, Component)]
pub struct ElseExpr;

derive_node!(IfExpr {
    relations = [
        children Exprs where tag = Expr,
        children App where tag = Expr,
        children Lambda where tag = Expr,
        children IfExpr where tag = Expr,
        children BinExpr where tag = Expr,
        children UnExpr where tag = Expr,
        children Ref where tag = Expr,
        children Literal where tag = Expr,
        children Tuple where tag = Expr,

        child Exprs,
        children ElifExpr,
        optional ElseExpr,
    ],
    properties = []
});
derive_node!(ElifExpr {
    relations = [
        children Exprs where tag = Expr,
        children App where tag = Expr,
        children Lambda where tag = Expr,
        children IfExpr where tag = Expr,
        children BinExpr where tag = Expr,
        children UnExpr where tag = Expr,
        children Ref where tag = Expr,
        children Literal where tag = Expr,
        children Tuple where tag = Expr,

        child Exprs,
    ],
    properties = []
});
derive_node!(ElseExpr {
    relations = [child Exprs,],
    properties = []
});

impl FromSyntax for IfExpr {
    type Syntax = kodept_rlt::prelude::IfExpr;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, IfExpr).with_children(source, pool, |scope| {
            scope.choose(Unit, [&node.condition]);
            scope.many::<ElifExpr, _>(node.elif.as_ref());
            scope.many::<ElseExpr, _>(node.el.as_ref());
            unwrap_body::<_, _, NoTag>(&node.body, scope);
        })
    }
}

impl FromSyntax for ElifExpr {
    type Syntax = kodept_rlt::prelude::ElifExpr;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, ElifExpr).with_children(source, pool, |scope| {
            scope.choose(Unit, [&node.condition]);
            unwrap_body::<_, _, NoTag>(&node.body, scope);
        })
    }
}

impl FromSyntax for ElseExpr {
    type Syntax = kodept_rlt::prelude::ElseExpr;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, ElseExpr).with_children(source, pool, |scope| {
            unwrap_body(&node.body, scope);
        })
    }
}
