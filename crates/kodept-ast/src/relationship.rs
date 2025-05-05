use crate::arity::{Arity, Optional, Singular};
use crate::prelude::ASTNode;
use crate::syntax_tree::children::HasChild;
use bevy_ecs::component::{Component, ComponentId, HookContext, Immutable};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use bevy_ecs::relationship::{Relationship, RelationshipSourceCollection};
use bevy_ecs::world::DeferredWorld;
use smallvec::SmallVec;
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum ArityValue {
    Singular,
    Optional,
    Plural,
}

/// Describes the entity that acts like a parent node for this entity
///
/// This is the source of truth for the relationship,
/// and can be modified directly to change the target
///
/// Type Parameters:
/// * [T] - Associated with this relationship tag
/// * [A] - Degree of relationship
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
    parent: Entity,
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

/// Describes a parent entity for some node for any tag or arity
#[derive(Debug, Component)]
#[relationship(relationship_target = Children)]
#[repr(transparent)]
pub struct ChildOf(Entity);

const ALL_CHILDREN_BUFFER_SIZE: usize = 2;

/// Describes all child nodes for any tag or arity
#[derive(Debug, Component)]
#[relationship_target(relationship = ChildOf)]
pub struct Children(SmallVec<[Entity; ALL_CHILDREN_BUFFER_SIZE]>);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct RelationshipMetadata {
    forward_id: ComponentId,
    backward_id: ComponentId,
    arity: ArityValue,
    tag_id: TypeId,
    tag_name: &'static str,
}

#[derive(Debug, Resource, Default)]
pub(crate) struct NodeRelationships(HashSet<RelationshipMetadata>);

pub trait NodeRelationship<Child, Tag> {
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

impl<T, U, Tag> NodeRelationship<U, Tag> for T
where
    T: HasChild<U, Tag> + ASTNode,
    U: ASTNode,
    Tag: 'static + Send + Sync,
{
    type Relationship = ContainedBy<Tag, T::Arity>;
}

impl<T, A> From<Entity> for ContainedBy<T, A>
where
    T: Send + Sync + 'static,
    A: Arity,
{
    fn from(value: Entity) -> Self {
        ContainedBy {
            parent: value,
            _phantom: PhantomData,
        }
    }
}

impl<T, A> Deref for ContainedBy<T, A>
where
    T: Send + Sync + 'static,
    A: Arity,
{
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl<T, A> DerefMut for ContainedBy<T, A>
where
    T: Send + Sync + 'static,
    A: Arity,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parent
    }
}

impl<T, A, C> AsRef<C> for Contains<T, A>
where
    T: Send + Sync + 'static,
    A::Collection: AsRef<C>,
    A: Arity,
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

impl Deref for ChildOf {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChildOf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a Children {
    type Item = Entity;
    type IntoIter =
        <SmallVec<[Entity; ALL_CHILDREN_BUFFER_SIZE]> as RelationshipSourceCollection>::SourceIter<
            'a,
        >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
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
            parent: self.parent,
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
        let forward_id = world.component_id::<Contains<T, A>>().unwrap();
        let metadata = RelationshipMetadata {
            forward_id,
            backward_id: ctx.component_id,
            arity: A::VALUE,
            tag_id: TypeId::of::<T>(),
            tag_name: std::any::type_name::<T>(),
        };

        if !world.get_resource::<NodeRelationships>().is_some_and(|it| it.0.contains(&metadata)) {
            if let Some(mut relationships) = world.get_resource_mut::<NodeRelationships>() {
                relationships.0.insert(metadata);
            } else {
                world.commands().insert_resource(NodeRelationships(HashSet::from([
                    metadata
                ])));
            }
        }

        // TODO: slow code ahead
        let (fetcher, mut commands) = world.entities_and_commands();
        let this = fetcher.get(ctx.entity).unwrap().get::<Self>().unwrap();
        commands.entity(this.parent).add_one_related::<ChildOf>(ctx.entity);
    }

    fn on_remove_hook(mut world: DeferredWorld, ctx: HookContext) {
        let mut commands = world.commands();
        commands.entity(ctx.entity).remove::<ChildOf>();
    }
}
