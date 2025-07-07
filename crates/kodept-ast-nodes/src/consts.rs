use bevy_ecs::name::Name;
use bevy_ecs::{bundle::Bundle, component::Component};
use kodept_ast::{
    derive_node, prelude::FromSyntax, properties::SourceSpan, relation,
    syntax_tree::prelude::ASTBuilder,
};
use kodept_rlt::exported::{Located, SpanBounds};
use kodept_rlt::prelude::{BodiedFunction, TopLevelNode};

use crate::Either;
use crate::{
    function::FuncDecl,
    top_level::{EnumDecl, StructDecl},
};

#[derive(Debug, Component)]
pub enum Const {
    Value,
    Enum,
    Struct,
    Fn
}

derive_node!(Const {
    properties = [ require Name, ]
});

relation!(Const => optional StructDecl);
relation!(Const => optional EnumDecl);
relation!(Const => optional FuncDecl);

impl FromSyntax<TopLevelNode> for Const {
    type Bundle = impl Bundle;

    fn from_syntax(
        node: &TopLevelNode,
        source: impl kodept_ast::prelude::CodeHolder,
    ) -> Self::Bundle {
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
        ASTBuilder::new(value)
            .with_dyn_child(node, source, |node, spawner, source| {
                match node {
                    TopLevelNode::Enum(x) => spawner.spawn::<_, EnumDecl, _>(x, source, Either::v31),
                    TopLevelNode::Struct(x) => spawner.spawn::<_, StructDecl, _>(x, source, Either::v32),
                    TopLevelNode::BodiedFunction(x) => spawner.spawn::<_, FuncDecl, _>(x, source, Either::v33),
                }
            })
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .build()
    }
}

impl FromSyntax<BodiedFunction> for Const {
    type Bundle = impl Bundle;

    fn from_syntax(node: &BodiedFunction, source: impl kodept_ast::prelude::CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(Const::Fn)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .with_child::<_, FuncDecl, _>(node, source)
            .build()
    }
}
