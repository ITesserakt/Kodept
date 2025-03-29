use std::ops::{Deref, DerefMut};
use bevy_ecs::prelude::{Component, Entity};

pub(crate) mod builder;
pub(crate) mod references;
pub(crate) mod storage;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub(super) enum Visibility {
    #[default]
    Private,
}

/// Attaches to the ast nodes and points to appropriate enclosing scope
#[derive(Debug, Component)]
#[relationship(relationship_target = Scoping)]
pub(crate) struct Scoped(Entity);

/// Describes a set of ast nodes that belongs to this scope
#[derive(Debug, Component)]
#[relationship_target(relationship = Scoped)]
pub(crate) struct Scoping(Vec<Entity>);

impl Deref for Scoped {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scoped {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
