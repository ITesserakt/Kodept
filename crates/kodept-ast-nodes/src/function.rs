use crate::expression::Exprs;
use crate::types::{NonTyParam, ProdTy, Ty, TyParam};
use crate::utils::{unwrap_body, unwrap_parameter, unwrap_type};
use bevy_ecs::prelude::{Bundle, Component, Name};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude::BodiedFunction;

#[derive(Debug, PartialEq, Component)]
pub struct FuncDecl;

derive_node!(FuncDecl {
    properties = [ require Name, ]
});
relation!(FuncDecl => either ReturnType(optional Ty));
relation!(FuncDecl => or ReturnType(optional ProdTy));
relation!(FuncDecl => either Body(child Exprs));
relation!(FuncDecl => either Params(children TyParam));
relation!(FuncDecl => or Params(children NonTyParam));

impl FromSyntax<BodiedFunction> for FuncDecl {
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(
        node: &BodiedFunction,
        source: impl CodeHolder,
    ) -> Result<Self::Bundle, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        Ok(ASTBuilder::new(FuncDecl)
            .with_dyn_child(&*node.body, source, unwrap_body)?
            .with_dyn_children(&node.return_type, |(_, it), spawner| {
                unwrap_type(it, spawner, source)
            })?
            .with_opt_dyn_children(
                node.params.as_ref().map(|it| it.inner.as_ref()),
                |it, spawner| unwrap_parameter(it, spawner, source),
            )?
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .build())
    }
}
