use crate::function::Func;
use crate::top_level::{EnumDecl, StructDecl};
use bevy_ecs::prelude::Component;
use kodept_ast::derive_node;
use kodept_ast::properties::Name;

/// Compile-time defined values or types
#[derive(Debug, PartialEq, Component)]
pub struct Const;

derive_node!(Const {
    relations = [
        optional EnumDecl,
        optional StructDecl,
        optional Func,
    ],
    properties = [require Name,]
});
