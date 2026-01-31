use crate::relationship::internal::ContainedBy;
use crate::syntax_tree::children::Family;
use bevy_ecs::component::{ComponentId, Immutable};
use bevy_ecs::prelude::Resource;
use bevy_ecs::relationship::Relationship;
use std::any::TypeId;
use std::collections::HashSet;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
// Describes the multiplicity of a relationship between AST node entities.
///
/// - `Singular`: exactly one related node (one-to-one).
/// - `Optional`: zero or one related node (zero-or-one).
/// - `Plural`: zero or more related nodes (zero-to-many).
///
/// This enum is used at runtime to register and query relationship metadata.
pub enum ArityValue {
    Singular,
    Optional,
    Plural,
}

mod internal {
    use crate::arity::{Arity, Optional, Singular};
    use crate::export::Component;
    use crate::relationship::{NodeRelationships, RelationshipMetadata};
    use bevy_ecs::entity::Entity;
    use bevy_ecs::lifecycle::HookContext;
    use bevy_ecs::prelude::{ChildOf, World};
    use bevy_ecs::relationship::{Relationship, RelationshipSourceCollection};
    use bevy_ecs::world::DeferredWorld;
    use std::any::TypeId;
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};

    /// Describes the entity that acts like a parent node for this entity
    ///
    /// This is the source of truth for the relationship,
    /// and can be modified directly to change the target
    ///
    /// Type Parameters:
    /// * [T] - relationship tag type (associates this relationship to a semantic tag).
    /// * [A] - arity marker (determines singular/optional/plural relationship).
    #[derive(Debug, Component)]
    #[relationship(relationship_target = Contains<T, A>)]
    #[component(on_add = ContainedBy::<T, A>::on_add_hook)]
    #[component(on_remove = ContainedBy::<T, A>::on_remove_hook)]
    #[repr(transparent)]
    pub struct ContainedBy<T, A>
    where
        T: Send + Sync + 'static,
        A: Arity,
    {
        #[relationship]
        value: Entity,
        _phantom: PhantomData<(T, A)>,
    }

    /// Describes all entities that are children for this entity
    ///
    /// Type parameters:
    /// * [T] - Associated with this relationship tag
    /// * [A] - Degree of relationship
    #[derive(Debug, Component)]
    #[relationship_target(relationship = ContainedBy<T, A>, linked_spawn)]
    #[repr(transparent)]
    pub struct Contains<T, A>
    where
        T: Send + Sync + 'static,
        A: Arity,
    {
        #[relationship]
        nodes: A::Collection,
        _phantom: PhantomData<(T, A)>,
    }

    impl<T, A> Deref for ContainedBy<T, A>
    where
        T: Send + Sync + 'static,
        A: Arity,
    {
        type Target = Entity;

        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }

    impl<T, A> DerefMut for ContainedBy<T, A>
    where
        T: Send + Sync + 'static,
        A: Arity,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.value
        }
    }

    impl<T, A, C> AsRef<C> for Contains<T, A>
    where
        T: Send + Sync + 'static,
        A::Collection: AsRef<C>,
        A: Arity,
        C: ?Sized,
    {
        fn as_ref(&self) -> &C {
            self.nodes.as_ref()
        }
    }

    impl<'a, T, A> IntoIterator for &'a Contains<T, A>
    where
        T: Send + Sync + 'static,
        A: Arity,
    {
        type Item = Entity;
        type IntoIter = <A::Collection as RelationshipSourceCollection>::SourceIter<'a>;

        fn into_iter(self) -> Self::IntoIter {
            self.nodes.iter()
        }
    }

    impl<T> Clone for Contains<T, Singular>
    where
        T: Send + Sync + 'static,
    {
        fn clone(&self) -> Self {
            Self {
                nodes: self.nodes,
                _phantom: Default::default(),
            }
        }
    }

    impl<T> Clone for Contains<T, Optional>
    where
        T: Send + Sync + 'static,
    {
        fn clone(&self) -> Self {
            Self {
                nodes: self.nodes,
                _phantom: Default::default(),
            }
        }
    }

    impl<T, A> Clone for ContainedBy<T, A>
    where
        T: Send + Sync + 'static,
        A: Arity,
    {
        fn clone(&self) -> Self {
            Self {
                value: self.value,
                _phantom: Default::default(),
            }
        }
    }

    impl<T, A> ContainedBy<T, A>
    where
        T: Send + Sync + 'static,
        A: Arity,
    {
        fn on_add_hook(mut world: DeferredWorld, ctx: HookContext) {
            let backward_id = ctx.component_id;
            world.commands().queue(move |world: &mut World| {
                let forward_id =
                    world.register_component::<<Self as Relationship>::RelationshipTarget>();
                let metadata = RelationshipMetadata {
                    forward_id,
                    backward_id,
                    arity: A::VALUE,
                    tag_id: TypeId::of::<T>(),
                    tag_name: std::any::type_name::<T>(),
                };
                let mut relationships = world.get_resource_or_init::<NodeRelationships>();

                relationships.0.insert(metadata);
                let mut this = world.entity_mut(ctx.entity);
                let parent = this.get::<Self>().unwrap();
                this.insert(ChildOf(parent.value));
            });
        }

        fn on_remove_hook(mut world: DeferredWorld, ctx: HookContext) {
            let mut commands = world.commands();
            commands.entity(ctx.entity).remove::<ChildOf>();
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
/// Metadata describing a relationship between AST nodes.
///
/// This struct stores runtime information required to register, identify and
/// query relationships between node entities. It is created when relationship
/// components are added and stored in the `NodeRelationships` resource.
pub struct RelationshipMetadata {
    /// Component id of the forward relationship component.
    forward_id: ComponentId,
    /// Component id of the backward relationship component.
    backward_id: ComponentId,
    /// Arity value describing multiplicity of the relationship.
    arity: ArityValue,
    /// TypeId of the tag associated with this relationship.
    tag_id: TypeId,
    /// Static string name of the tag type.
    tag_name: &'static str,
}

#[derive(Debug, Resource, Default)]
pub(crate) struct NodeRelationships(HashSet<RelationshipMetadata>);

pub trait NodeRelationship<Tag, Arity> {
    type Relationship: Relationship<Mutability = Immutable>;
}

impl RelationshipMetadata {
    pub fn tag_name(&self) -> &'static str {
        self.tag_name
    }

    pub fn is_empty_tag(&self) -> bool {
        self.tag_id == TypeId::of::<()>()
    }

    pub fn arity(&self) -> ArityValue {
        self.arity
    }

    pub fn forward_component_id(&self) -> ComponentId {
        self.forward_id
    }

    pub fn backward_component_id(&self) -> ComponentId {
        self.backward_id
    }
}

impl<'a> IntoIterator for &'a NodeRelationships {
    type Item = RelationshipMetadata;
    type IntoIter = std::iter::Cloned<std::collections::hash_set::Iter<'a, RelationshipMetadata>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().cloned()
    }
}

impl<T, Tag> NodeRelationship<Tag, T::Arity> for T
where
    T: Family<Tag>,
    Tag: 'static + Send + Sync,
{
    type Relationship = ContainedBy<Tag, T::Arity>;
}
