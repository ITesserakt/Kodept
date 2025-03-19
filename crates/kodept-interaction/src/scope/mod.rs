use bevy_ecs::entity::hash_map::EntityHashMap;
use bevy_ecs::prelude::{Entity, Resource};
use kodept_ast::prelude::Erase;

pub(crate) mod builder;
pub(crate) mod references;
pub(crate) mod storage;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub(super) enum Visibility {
    #[default]
    Private,
}

#[derive(Debug, Resource)]
pub(crate) struct ScopeMapping {
    /// Mapping from entity to its enclosing scope entity
    enclosing_scopes_mapping: EntityHashMap<Entity>,
    root_entity: Entity
}

impl ScopeMapping {
    pub(crate) fn new(mapping: EntityHashMap<Entity>, root_entity: impl Erase<Entity>) -> Self {
        Self {
            enclosing_scopes_mapping: mapping,
            root_entity: root_entity.erase(),
        }
    }
    
    pub(crate) fn enclosing_scope_id(&self, entity: impl Erase<Entity>) -> Entity {
        *self.enclosing_scopes_mapping.get(&entity.erase()).unwrap()
    }
    
    pub(crate) fn root_scope_id(&self) -> Entity { 
        self.enclosing_scopes_mapping[&self.root_entity]
    }
}
