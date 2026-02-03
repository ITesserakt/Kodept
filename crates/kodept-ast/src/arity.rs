use crate::node_id::NodeId;
use crate::relationship::ArityValue;
use crate::syntax_tree::children::Nothing;
use bevy_ecs::entity::{EntityMapper, MapEntities};
use bevy_ecs::prelude::Entity;
use bevy_ecs::relationship::RelationshipSourceCollection;
use private::Sealed;
use smallvec::SmallVec;

mod private {
    pub trait Sealed {}
}

pub trait Arity: Sealed + 'static + Send + Sync {
    type Collection: RelationshipSourceCollection + MapEntities + Send + Sync + 'static;

    const VALUE: ArityValue;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Option(Entity);

/// Describes that parent must have no children of that type
/// Zero-to-one relationship
#[derive(Debug)]
pub struct Empty;

/// Describes that parent must have a single child of that type
/// One-to-one relationship
#[derive(Debug)]
pub struct Singular;

/// Describes that parent may not have any child of that type
/// Zero or one-to-one relationship
#[derive(Debug)]
pub struct Optional;

/// Describes that parent may have multiple children of that type
/// Many-to-one relationship
#[derive(Debug)]
pub struct Plural;

impl Sealed for Empty {}
impl Arity for Empty {
    type Collection = Nothing;
    const VALUE: ArityValue = ArityValue::Empty;
}
impl Sealed for Singular {}
impl Arity for Singular {
    type Collection = NodeId;
    const VALUE: ArityValue = ArityValue::Singular;
}
impl Sealed for Optional {}
impl Arity for Optional {
    type Collection = Option;
    const VALUE: ArityValue = ArityValue::Optional;
}
impl Sealed for Plural {}
impl Arity for Plural {
    type Collection = SmallVec<[Entity; 2]>;
    const VALUE: ArityValue = ArityValue::Plural;
}

impl RelationshipSourceCollection for Option {
    type SourceIter<'a> = std::option::IntoIter<Entity>;

    fn new() -> Self {
        Option(Entity::PLACEHOLDER)
    }

    fn with_capacity(_: usize) -> Self {
        Self(Entity::PLACEHOLDER)
    }

    fn reserve(&mut self, _: usize) {}

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
        if self.0 == Entity::PLACEHOLDER { 0 } else { 1 }
    }

    fn clear(&mut self) {
        *self = Option(Entity::PLACEHOLDER);
    }

    fn shrink_to_fit(&mut self) {}

    fn is_empty(&self) -> bool {
        self.0 == Entity::PLACEHOLDER
    }

    fn extend_from_iter(&mut self, entities: impl IntoIterator<Item = Entity>) {
        if let Some(first) = entities.into_iter().next() {
            self.0 = first;
        }
    }
}

impl MapEntities for Option {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.into_inner().map_entities(entity_mapper)
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

impl RelationshipSourceCollection for Nothing {
    type SourceIter<'a> = std::iter::Empty<Entity>;

    fn new() -> Self {
        unreachable!("Children with `Empty` arity cannot exist")
    }

    #[inline]
    fn with_capacity(_: usize) -> Self {
        Self::new()
    }

    #[inline]
    fn reserve(&mut self, _: usize) {}

    #[inline]
    fn add(&mut self, _: Entity) -> bool {
        false
    }

    #[inline]
    fn remove(&mut self, _: Entity) -> bool {
        false
    }

    #[inline]
    fn iter(&self) -> Self::SourceIter<'_> {
        std::iter::empty()
    }

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn clear(&mut self) {}

    #[inline]
    fn shrink_to_fit(&mut self) {}

    #[inline]
    fn extend_from_iter(&mut self, _: impl IntoIterator<Item = Entity>) {}
}

impl MapEntities for Nothing {
    fn map_entities<E: EntityMapper>(&mut self, _: &mut E) {}
}
