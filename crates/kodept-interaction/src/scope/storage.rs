use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;

#[derive(Debug, Component, Hash, Eq, PartialEq)]
pub struct Scope {
    /// Root entity for this scope
    pub start_from: Entity,
    /// Defines whether variables inside the scope are visible outside
    pub is_anonymous: bool,
}

pub enum ScopeV2 {
    
}

impl Scope {
    pub fn new(start_from: Entity, is_anonymous: bool) -> Self {
        Self {
            start_from,
            is_anonymous,
        }
    }
}
