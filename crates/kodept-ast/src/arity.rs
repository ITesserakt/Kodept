use bevy_ecs::entity::{VisitEntities, VisitEntitiesMut};
use bevy_ecs::prelude::Entity;
use bevy_ecs::relationship::RelationshipSourceCollection;
use private::Sealed;
use crate::relationship::ArityValue;

mod private {
    pub trait Sealed {}
}

pub trait Arity: Sealed + 'static + Send + Sync {
    type Collection: RelationshipSourceCollection
    + Sync
    + Send
    + VisitEntities
    + VisitEntitiesMut;
    
    const VALUE: ArityValue;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Option(Entity);

/// Describes that parent must have single child of that type
/// One-to-one relationship
pub struct Singular;

/// Describes that parent may not have any child of that type
/// Zero or one-to-one relationship
pub struct Optional;

/// Describes that parent may have multiple children of that type
/// Many-to-one relationship
pub struct Plural;

impl Sealed for Singular {}
impl Arity for Singular {
    type Collection = Entity;
    const VALUE: ArityValue = ArityValue::Singular;
}
impl Sealed for Optional {}
impl Arity for Optional {
    type Collection = Option;
    const VALUE: ArityValue = ArityValue::Optional;
}
impl Sealed for Plural {}
impl Arity for Plural {
    type Collection = Vec<Entity>;
    const VALUE: ArityValue = ArityValue::Plural;
}

impl RelationshipSourceCollection for Option {
    type SourceIter<'a> = std::option::IntoIter<Entity>;

    fn with_capacity(_: usize) -> Self {
        Self(Entity::PLACEHOLDER)
    }

    fn add(&mut self, entity: Entity) -> bool {
        if self.0 == Entity::PLACEHOLDER {
            *self = Option(entity);
            true
        } else {
            false
        }
    }

    fn remove(&mut self, entity: Entity) -> bool {
        if self.0 == entity {
            *self = Option(Entity::PLACEHOLDER);
            true
        } else {
            false
        }
    }

    fn iter(&self) -> Self::SourceIter<'_> {
        if self.0 != Entity::PLACEHOLDER {
            Some(self.0).into_iter()
        } else {
            None.into_iter()
        }
    }

    fn len(&self) -> usize {
        if self.0 == Entity::PLACEHOLDER {
            0
        } else {
            1
        }
    }

    fn clear(&mut self) {
        *self = Option(Entity::PLACEHOLDER);
    }
}

impl Option {
    pub fn into_inner(self) -> std::option::Option<Entity> {
        if self.0 == Entity::PLACEHOLDER {
            None
        } else {
            Some(self.0)
        }
    }
}

impl From<Option> for std::option::Option<Entity> {
    fn from(value: Option) -> Self {
        value.into_inner()
    }
}

impl VisitEntities for Option {
    fn visit_entities<F: FnMut(Entity)>(&self, mut f: F) {
        f(self.0)
    }
}

impl VisitEntitiesMut for Option {
    fn visit_entities_mut<F: FnMut(&mut Entity)>(&mut self, mut f: F) {
        f(&mut self.0)
    }
}