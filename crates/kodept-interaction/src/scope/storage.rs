use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;

#[derive(Debug, Component, Hash, Eq, PartialEq)]
pub(super) struct Scope {
    /// Root entity for this scope
    pub start_from: Entity,
    /// Defines whether symbols inside the scope are visible outside
    pub is_anonymous: bool,
    /// Defines whether inner scopes may access symbols of this scope
    pub opaque: bool,
}

impl Scope {
    pub(super) fn new(start_from: Entity, is_anonymous: bool) -> Self {
        Self {
            start_from,
            is_anonymous,
            opaque: false,
        }
    }

    pub(super) fn opaque(self, opaque: bool) -> Self {
        Self { opaque, ..self }
    }
}
