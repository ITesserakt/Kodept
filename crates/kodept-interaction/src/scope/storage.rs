use crate::scope::Scoped;
use bevy_ecs::entity::hash_map::EntityHashMap;
use bevy_ecs::prelude::{ChildOf, Commands, Component, Entity, Local};
use bevy_ecs::system::SystemParam;
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast::properties::Name;

#[derive(Debug, Component, Hash, Eq, PartialEq)]
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

#[derive(SystemParam)]
pub struct ScopeBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    scope_mapping: Local<'s, EntityHashMap<Entity>>,
}

impl<'w, 's> ScopeBuilder<'w, 's> {
    pub fn allocate_scope(
        &mut self,
        start_from: impl Erase,
        name: Option<Name>,
        is_anonymous: bool,
        is_opaque: bool,
    ) -> Entity {
        let id = start_from.erase();

        let mut commands = if let Some(name) = name {
            self.commands.spawn((
                name,
                Scope {
                    start_from: id,
                    is_anonymous,
                    opaque: is_opaque,
                },
            ))
        } else {
            self.commands.spawn(Scope {
                start_from: id,
                is_anonymous,
                opaque: is_opaque,
            })
        };

        self.scope_mapping.insert(id.entity(), commands.id());
        commands.add_one_related::<Scoped>(id.entity()).id()
    }

    pub fn link_scopes(&mut self, scope_id: Entity, parent_scope_id: Entity) {
        self.commands
            .entity(scope_id)
            .insert(ChildOf(parent_scope_id));
    }

    pub fn get_enclosing_scope(&self, node_id: NodeId) -> Option<Entity> {
        self.scope_mapping.get(&node_id.entity()).copied()
    }

    pub fn set_enclosing_scope(&mut self, node_id: NodeId, scope_id: Entity) {
        self.scope_mapping.insert(node_id.entity(), scope_id);
        self.commands
            .entity(node_id.entity())
            .insert(Scoped(scope_id));
    }
}
