use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use bevy_ecs::prelude::Entity;

pub struct NodeId<T = ()> {
    entity: Entity,
    _phantom: PhantomData<T>
}

pub type AnyNodeId = NodeId;

impl<T> NodeId<T> {
    pub const NULL: Self = Self {
        entity: Entity::PLACEHOLDER,
        _phantom: PhantomData,
    };

    pub(crate) const fn cast<U>(self) -> NodeId<U> {
        NodeId {
            entity: self.entity,
            _phantom: PhantomData,
        }
    }
    
    pub(crate) const fn as_inner(&self) -> Entity {
        self.entity
    }
    
    pub(crate) const fn from_inner(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData
        }
    }
}

impl<T> Hash for NodeId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entity.hash(state)
    }
}

impl<T> Display for NodeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.entity.index(), self.entity.generation())
    }
}

impl<T> Copy for NodeId<T> {}

impl<T> Clone for NodeId<T> {
    fn clone(&self) -> Self {
        *self
    }
}
