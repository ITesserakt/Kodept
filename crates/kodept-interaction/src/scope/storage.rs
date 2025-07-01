use crate::symbol::table::SymbolTable;
use bevy_ecs::prelude::Component;
use kodept_ast::prelude::{Erase, NodeId};

#[derive(Debug, Component, Hash, Eq, PartialEq)]
#[component(immutable)]
#[require(SymbolTable)]
pub(crate) struct Scope {
    /// Root entity for this scope
    pub start_from: NodeId,
}

impl Scope {
    pub(super) fn new(start_from: impl Erase) -> Self {
        Self {
            start_from: start_from.erase(),
        }
    }
}
