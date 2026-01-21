use bevy_ecs::component::Component;
use bevy_ecs::name::Name;
use bevy_ecs::relationship::Relationship;
use kodept_ast::experimental::{AstBuilder, FromSyntax};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::syntax_tree::experimental::{Buffer, GenericSpawnContext, SpawnedIn};
use kodept_ast::{derive_node, properties::SourceSpan, relation};
use kodept_rlt::exported::{Located, SpanBounds};
use kodept_rlt::prelude::{BodiedFunction, TopLevelNode};

use crate::v2::function::FuncDecl;
use crate::v2::top_level::{EnumDecl, StructDecl};

#[derive(Debug, PartialEq, Component)]
pub enum Const {
    Value,
    Enum,
    Struct,
    Fn,
}

derive_node!(Const {
    properties = [ require Name, ]
});

relation!(Const => optional StructDecl);
relation!(Const => optional EnumDecl);
relation!(Const => optional FuncDecl);

impl FromSyntax<TopLevelNode> for Const {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &TopLevelNode,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let id_point = match node {
            TopLevelNode::Enum(x) => x.id().location(),
            TopLevelNode::Struct(x) => x.id.location(),
            TopLevelNode::BodiedFunction(x) => x.id.location(),
        };
        let name = source.get_chunk(id_point);
        let value = match node {
            TopLevelNode::Enum(_) => Const::Enum,
            TopLevelNode::Struct(_) => Const::Struct,
            TopLevelNode::BodiedFunction(_) => Const::Fn,
        };
        let mut builder = AstBuilder::new(value)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);
        match node {
            TopLevelNode::Enum(x) => Ok(builder.with_child::<_, EnumDecl, _>(x, source)?.finish()),
            TopLevelNode::Struct(x) => {
                Ok(builder.with_child::<_, StructDecl, _>(x, source)?.finish())
            }
            TopLevelNode::BodiedFunction(x) => {
                Ok(builder.with_child::<_, FuncDecl, _>(x, source)?.finish())
            }
        }
    }
}

impl FromSyntax<BodiedFunction> for Const {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &BodiedFunction,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        AstBuilder::new(Const::Fn)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner)
            .with_child::<_, FuncDecl, _>(node, source)
            .map(|it| it.finish())
    }
}
