use crate::types::{Params, ProdTy, Ty};
use crate::utils::{unwrap_body, wrap_params};
use crate::Unit;
use kodept_ast::{derive_node, relation};
use kodept_ast::external::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::Name;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_rlt::prelude::BodiedFunction;
use crate::expression::Exprs;

#[derive(Debug, PartialEq, Component)]
pub struct Func;

derive_node!(Func {
    properties = [require Name,]
});
relation!(Func => optional Ty);
relation!(Func => optional ProdTy);
relation!(Func => child Exprs);
relation!(Func => child Params);

impl FromSyntax for Func {
    type Syntax = BodiedFunction;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);

        ASTBuilder::new(pool, Func)
            .with_property(Name(name))
            .with_children(source, pool, move |scope| {
                scope.choose(Unit, node.return_type.as_ref().map(|it| &it.1));
                if let Some(params) = node.params.as_ref() {
                    wrap_params(node, &params.inner, scope);
                }
                unwrap_body(&node.body, scope);
            })
    }
}
