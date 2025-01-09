use kodept_ast::derive_node;
use kodept_ast::external::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_core::structure::rlt;

#[derive(Debug, PartialEq, Component)]
pub struct IfExpr;

derive_node!(IfExpr);

impl FromSyntax for IfExpr {
    type Syntax = rlt::IfExpr;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, IfExpr)
    }
}
