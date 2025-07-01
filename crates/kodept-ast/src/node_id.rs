use bevy_ecs::prelude::Entity;
use bevy_ecs::relationship::RelationshipSourceCollection;
use derive_more::Into;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use bevy_ecs::entity::{ContainsEntity, EntityEquivalent};

/// Ids that their associated type can be erased
pub trait Erase<Erased = NodeId> {
    fn erase(self) -> Erased;
}

impl Erase<NodeId> for Entity {
    #[inline]
    fn erase(self) -> NodeId {
        self.into()
    }
}

impl<T> Erase<NodeId> for NodeId<T> {
    #[inline]
    fn erase(self) -> NodeId {
        self.cast()
    }
}

impl<T: Erase> Erase<Entity> for T {
    fn erase(self) -> Entity {
        T::erase(self).entity
    }
}

#[derive(Into)]
pub struct NodeId<T = ()> {
    entity: Entity,
    #[into(ignore)]
    _phantom: PhantomData<T>,
}

impl<T> From<Entity> for NodeId<T> {
    fn from(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: Default::default(),
        }
    }
}

impl<T> NodeId<T> {
    pub const NULL: Self = Self {
        entity: Entity::PLACEHOLDER,
        _phantom: PhantomData,
    };

    pub fn cast<U>(self) -> NodeId<U> {
        NodeId {
            entity: self.entity,
            _phantom: PhantomData,
        }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }
}

impl<T> Clone for NodeId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeId<T> {}

impl<T> Deref for NodeId<T> {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T> DerefMut for NodeId<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<T> Debug for NodeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeId")
            .field("entity", &self.entity)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<T> PartialEq for NodeId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

impl<T> Eq for NodeId<T> {}

impl<T> PartialOrd for NodeId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.entity.partial_cmp(&other.entity)
    }
}

impl<T> Ord for NodeId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.entity.cmp(&other.entity)
    }
}

impl<T> Hash for NodeId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.entity.hash(state)
    }
}

impl<T> Display for NodeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Entity as Display>::fmt(&self.entity, f)
    }
}

impl<T> Borrow<Entity> for NodeId<T> {
    fn borrow(&self) -> &Entity {
        &self.entity
    }
}

impl<T> RelationshipSourceCollection for NodeId<T> {
    type SourceIter<'a>
        = std::option::IntoIter<Entity>
    where
        T: 'a;

    fn new() -> Self {
        Self::NULL
    }

    fn with_capacity(_capacity: usize) -> Self {
        Self::NULL
    }

    fn reserve(&mut self, additional: usize) {
        self.entity.reserve(additional)
    }

    fn add(&mut self, entity: Entity) -> bool {
        self.entity.add(entity)
    }

    fn remove(&mut self, entity: Entity) -> bool {
        self.entity.remove(entity)
    }

    fn iter(&self) -> Self::SourceIter<'_> {
        self.entity.iter()
    }

    fn len(&self) -> usize {
        self.entity.len()
    }

    fn clear(&mut self) {
        self.entity.clear()
    }

    fn shrink_to_fit(&mut self) {
        self.entity.shrink_to_fit()
    }
}

impl<T> ContainsEntity for NodeId<T> {
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[allow(unsafe_code)]
unsafe impl<T> EntityEquivalent for NodeId<T> {}
