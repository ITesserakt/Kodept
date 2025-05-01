use crate::top_level::{EnumDecl, StructDecl};
use bevy_ecs::prelude::Component;
use kodept_ast::{derive_node, relation};
use kodept_ast::properties::Name;
use crate::function::Func;

/// Compile-time defined values or types
#[derive(Debug, PartialEq, Component)]
pub struct Const;

derive_node!(Const {
    properties = [require Name,]
});
relation!(Const => optional EnumDecl);
relation!(Const => optional StructDecl);
relation!(Const => optional Func);
