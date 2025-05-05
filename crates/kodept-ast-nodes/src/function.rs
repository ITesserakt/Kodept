use crate::expression::Exprs;
use crate::types::{Params, ProdTy, Ty};
use crate::utils::{unwrap_body, unwrap_parameter, unwrap_type};
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude::BodiedFunction;
use std::convert::identity;

#[derive(Debug, PartialEq, Component)]
pub struct Func;

derive_node!(Func {
    properties = [require Name,]
});
relation!(Func => optional Ty);
relation!(Func => optional ProdTy);
relation!(Func => child Exprs);
relation!(Func => either P(optional Params));

impl FromSyntax<BodiedFunction> for Func {
    type Bundle = impl Bundle;

    fn from_syntax(node: &BodiedFunction, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(&node.id);

        ASTBuilder::new(Func)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_children(&node.return_type, |(_, it), spawner| {
                unwrap_type(it, spawner, source)
            })
            .with_opt_dyn_child(node.params.as_ref(), source, |it, spawner, source| {
                spawner.spawn_raw(
                    ASTBuilder::new(Params)
                        .with_property(SourceSpan(it.left.0 + it.right.0))
                        .with_dyn_children(it.inner.as_ref(), |it, spawner| {
                            unwrap_parameter(it, spawner, source)
                        }),
                    node,
                    identity,
                )
            })
            .with_dyn_child(node.body.as_ref(), source, unwrap_body)
            .build()
    }
}
