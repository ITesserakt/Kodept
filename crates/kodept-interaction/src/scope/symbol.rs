use std::sync::OnceLock;
use bevy_ecs::prelude::Component;
use kodept_inference::r#type::PolymorphicType;

#[derive(Debug, PartialEq)]
pub enum SymbolKind {
    Type,
    Variable,
    Parameter,
    Function
}

#[derive(Debug, PartialEq, Component)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub ty: OnceLock<PolymorphicType>
}
