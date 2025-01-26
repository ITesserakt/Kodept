use crate::scope::storage::Scope;
use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::{Entity, Query, Resource};

pub(crate) mod builder;
mod references;
mod storage;
pub(crate) mod symbol;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Private,
    Public
}

#[derive(Debug, Resource)]
pub(crate) struct ScopeMapping {
    /// Mapping from entity to its enclosing scope entity
    pub enclosing_scopes_mapping: EntityHashMap<Entity>,
}

impl ScopeMapping {
    pub(crate) fn enclosing_scope<'a>(
        &self,
        entity: Entity,
        scopes: &'a Query<&Scope>,
    ) -> (Entity, &'a Scope) {
        let scope_id = self.enclosing_scopes_mapping.get(&entity).unwrap();
        (*scope_id, scopes.get(*scope_id).unwrap())
    }

    pub(crate) fn enclosing_scope_id(&self, entity: Entity) -> Entity {
        *self.enclosing_scopes_mapping.get(&entity).unwrap()
    }
}
