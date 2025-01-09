use crate::expression::Exprs;
use crate::properties::{Param, Type};
use crate::types::{NonTyParam, ProdTy, TyName, TyParam};
use crate::utils::unwrap_body;
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, Str};
use kodept_rlt::BodiedFunction;

#[derive(Debug, PartialEq, Component)]
pub struct Func {
    pub name: Str,
}

derive_node!(Func {
    relations = [
        optional TyName where tag = Type,
        optional ProdTy where tag = Type,

        children TyParam where tag = Param,
        children NonTyParam where tag = Param,

        child Exprs,
    ],
    properties = []
});

impl FromSyntax for Func {
    type Syntax = BodiedFunction;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);

        ASTBuilder::new(pool, Func { name }).with_children(source, pool, move |scope| {
            scope.choose(Unit, node.return_type.as_ref().map(|it| &it.1));
            scope.maybe_choose(Unit, node.params.as_ref().map(|it| it.inner.as_ref()));
            unwrap_body(&node.body, scope);
        })
    }
}
