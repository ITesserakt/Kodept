use crate::v2::expression::Exprs;
use crate::v2::types::{NonTyParam, ProdTy, Ty, TyParam};
use crate::v2::Dispatcher;
use bevy_ecs::prelude::{Component, Name};
use bevy_ecs::relationship::Relationship;
use kodept_ast::experimental::{AstBuilder, FromSyntax};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::experimental::{Buffer, GenericSpawnContext, SpawnedIn};
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
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &BodiedFunction,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        let mut builder = AstBuilder::new(FuncDecl)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);
        builder.with_dispatch::<Dispatcher<_>, _, _>(node.body.as_ref(), source)?;
        if let Some((_, ret)) = &node.return_type {
            builder.with_dispatch::<Dispatcher<_>, _, _>(ret, source)?;
        }
        if let Some(params) = node.params.as_ref().map(|it| it.inner.as_ref()) {
            builder.with_dispatches::<Dispatcher<_>, _, _>(params, source)?;
        }

        Ok(builder.finish())
    }
}
