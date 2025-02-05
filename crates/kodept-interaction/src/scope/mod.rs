use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::{Entity, Resource};

pub(crate) mod builder;
pub(crate) mod references;
mod storage;
pub(crate) mod symbol;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
enum Visibility {
    #[default]
    Private,
}

#[derive(Debug, Resource)]
struct ScopeMapping {
    /// Mapping from entity to its enclosing scope entity
    pub enclosing_scopes_mapping: EntityHashMap<Entity>,
}

impl ScopeMapping {
    pub(crate) fn enclosing_scope_id(&self, entity: Entity) -> Entity {
        *self.enclosing_scopes_mapping.get(&entity).unwrap()
    }
}
