use crate::symbol::table::SymbolTable;
use bevy_ecs::prelude::Component;
use kodept_ast::prelude::{Erase, NodeId};

#[derive(Debug, Component, Hash, Eq, PartialEq)]
#[require(SymbolTable)]
pub(crate) struct Scope {
    /// Root entity for this scope
    pub start_from: NodeId,
    /// Defines whether symbols inside the scope are visible outside
    pub is_anonymous: bool,
    /// Defines whether inner scopes may access symbols of this scope
    pub opaque: bool,
}

impl Scope {
    pub(super) fn new(start_from: impl Erase, is_anonymous: bool) -> Self {
        Self {
            start_from: start_from.erase(),
            is_anonymous,
            opaque: false,
        }
    }

    pub(super) fn opaque(self, opaque: bool) -> Self {
        Self { opaque, ..self }
    }
}
