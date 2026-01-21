use crate::function::FuncDecl;
use crate::types::TyParam;
use bevy_ecs::prelude::Component;
use bevy_ecs::relationship::Relationship;
use kodept_ast::experimental::{AstBuilder, FromSyntax};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::syntax_tree::experimental::{Buffer, GenericSpawnContext, SpawnedIn};
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::new_types::TypeName;
use kodept_rlt::prelude::{Enum, Struct};
use std::convert::Infallible;

#[derive(Debug, PartialEq, Component)]
pub enum EnumDecl {
    Stack,
    Heap,
}

#[derive(Debug, PartialEq, Component)]
pub struct StructDecl;

#[derive(Debug, PartialEq, Component)]
pub struct EnumConst;

derive_node!(EnumDecl {
    properties = [require Name,]
});
relation!(EnumDecl => children EnumConst);

derive_node!(StructDecl {
    properties = [require Name,]
});
relation!(StructDecl => children FuncDecl);
relation!(StructDecl => either P(children TyParam));

derive_node!(EnumConst {
    properties = [require Name,]
});

impl FromSyntax<Enum> for EnumDecl {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &Enum,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let (kind, id, rest) = match node {
            Enum::Stack { id, contents, .. } => (EnumDecl::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumDecl::Heap, id, contents),
        };
        let name = source.get_chunk_located(id);
        let mut builder = AstBuilder::new(kind)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);
        if let Some(rest) = rest {
            builder.with_children::<_, EnumConst, _>(rest.inner.as_ref(), source)?;
        }
        Ok(builder.finish())
    }
}

impl FromSyntax<Struct> for StructDecl {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &Struct,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        let mut builder = AstBuilder::new(StructDecl)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);

        if let Some(params) = &node.parameters {
            builder.with_children::<_, TyParam, _>(params.inner.as_ref(), source)?;
        }
        if let Some(body) = &node.body {
            builder.with_children::<_, FuncDecl, _>(body.inner.as_ref(), source)?;
        }

        Ok(builder.finish())
    }
}

impl FromSyntax<TypeName> for EnumConst {
    type Error = Infallible;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &TypeName,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(node);

        Ok(AstBuilder::new(EnumConst)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.0.into()))
            .spawn_in(spawner)
            .finish())
    }
}
