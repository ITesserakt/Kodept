use crate::arity::Arity;
use crate::prelude::ASTNode;
use crate::syntax_tree::children::HasChild;
use bevy_ecs::component::{
    Component, ComponentId, Components, ComponentsRegistrator, Mutable, RequiredComponents,
    StorageType,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::RelationshipTarget;
use bevy_ecs::relationship::{Relationship, RelationshipSourceCollection};
use dashmap::setref::multiple::RefMulti;
use dashmap::{DashMap, DashSet};
use std::any::TypeId;
use std::hash::RandomState;
use std::iter::Map;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;

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
#[derive(Debug)]
pub struct ContainedBy<T, A>(Entity, PhantomData<(T, A)>);

/// Describes all entities that are children for this entity
///
/// Type parameters:
/// * [T] - Associated with this relationship tag
/// * [A] - Degree of relationship
#[derive(Debug)]
pub struct Contains<T, A: Arity>(A::Collection, PhantomData<(T, A)>);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct RelationshipMetadata {
    forward_id: TypeId,
    backward_id: TypeId,
    arity: ArityValue,
    tag_id: TypeId,
    tag_name: &'static str,
}

impl RelationshipMetadata {
    fn new<T, U, Tag>() -> Self
    where
        T: HasChild<U, Tag>,
        U: ASTNode,
        Tag: 'static,
    {
        Self {
            forward_id: TypeId::of::<<T::Relationship as Relationship>::RelationshipTarget>(),
            backward_id: TypeId::of::<T::Relationship>(),
            arity: <T::Arity>::VALUE,
            tag_id: TypeId::of::<Tag>(),
            tag_name: std::any::type_name::<Tag>(),
        }
    }
}

static NODE_RELATIONSHIP_METADATA: LazyLock<DashSet<RelationshipMetadata>> =
    LazyLock::new(|| DashSet::new());

pub(crate) struct NodeRelationships;

pub trait NodeRelationship<Child, Tag> {
    type Relationship: Relationship<Mutability = Mutable>;

    fn register();
}

impl RelationshipMetadata {
    pub(crate) fn tag_name(&self) -> &'static str {
        self.tag_name
    }

    pub(crate) fn is_empty_tag(&self) -> bool {
        self.tag_id == TypeId::of::<()>()
    }

    pub(crate) fn arity(&self) -> ArityValue {
        self.arity
    }

    pub(crate) fn forward_component_id(&self, components: &Components) -> ComponentId {
        components.get_id(self.forward_id).unwrap()
    }

    pub(crate) fn backward_component_id(&self, components: &Components) -> ComponentId {
        components.get_id(self.backward_id).unwrap()
    }
}

impl IntoIterator for NodeRelationships {
    type Item = RelationshipMetadata;
    type IntoIter = Map<
        dashmap::iter_set::Iter<
            'static,
            RelationshipMetadata,
            RandomState,
            DashMap<RelationshipMetadata, ()>,
        >,
        fn(RefMulti<RelationshipMetadata>) -> RelationshipMetadata,
    >;

    fn into_iter(self) -> Self::IntoIter {
        NODE_RELATIONSHIP_METADATA.iter().map(|it| *it.key())
    }
}

impl<T, U, Tag> NodeRelationship<U, Tag> for T
where
    T: HasChild<U, Tag> + ASTNode,
    U: ASTNode,
    Tag: 'static + Send + Sync,
{
    type Relationship = ContainedBy<Tag, T::Arity>;

    fn register() {
        NODE_RELATIONSHIP_METADATA.insert(RelationshipMetadata::new::<T, U, Tag>());
    }
}

impl<T, A> From<Entity> for ContainedBy<T, A> {
    fn from(value: Entity) -> Self {
        ContainedBy(value, PhantomData)
    }
}

impl<T, A> Deref for ContainedBy<T, A> {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, A> DerefMut for ContainedBy<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, A: Arity, C> AsRef<C> for Contains<T, A>
where
    A::Collection: AsRef<C>,
{
    fn as_ref(&self) -> &C {
        self.0.as_ref()
    }
}

impl<'a, T, A> IntoIterator for &'a Contains<T, A>
where
    A: Arity,
{
    type Item = Entity;
    type IntoIter = <A::Collection as RelationshipSourceCollection>::SourceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T, A> Component for Contains<T, A>
where
    T: Send + Sync + 'static,
    A: Arity,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;
    fn on_replace() -> Option<bevy_ecs::component::ComponentHook> {
        Some(<Self as RelationshipTarget>::on_replace)
    }
    fn register_required_components(
        _requiree: ComponentId,
        components: &mut ComponentsRegistrator,
        _required_components: &mut RequiredComponents,
        _inheritance_depth: u16,
        recursion_check_stack: &mut Vec<ComponentId>,
    ) {
        bevy_ecs::component::enforce_no_required_components_recursion(
            components,
            recursion_check_stack,
        );
        let self_id = components.register_component::<Self>();
        recursion_check_stack.push(self_id);
        recursion_check_stack.pop();
    }
    fn clone_behavior() -> bevy_ecs::component::ComponentCloneBehavior {
        bevy_ecs::component::ComponentCloneBehavior::Custom(
            bevy_ecs::relationship::clone_relationship_target::<Self>,
        )
    }
}

impl<T, A> Component for ContainedBy<T, A>
where
    T: Send + Sync + 'static,
    A: Arity,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn on_insert() -> Option<bevy_ecs::component::ComponentHook> {
        Some(<Self as Relationship>::on_insert)
    }
    fn on_replace() -> Option<bevy_ecs::component::ComponentHook> {
        Some(<Self as Relationship>::on_replace)
    }
    fn register_required_components(
        _requiree: ComponentId,
        components: &mut ComponentsRegistrator,
        _required_components: &mut RequiredComponents,
        _inheritance_depth: u16,
        recursion_check_stack: &mut Vec<ComponentId>,
    ) {
        bevy_ecs::component::enforce_no_required_components_recursion(
            components,
            recursion_check_stack,
        );
        let self_id = components.register_component::<Self>();
        recursion_check_stack.push(self_id);
        recursion_check_stack.pop();
    }
    fn clone_behavior() -> bevy_ecs::component::ComponentCloneBehavior {
        use bevy_ecs::component::DefaultCloneBehaviorBase;
        (&&&bevy_ecs::component::DefaultCloneBehaviorSpecialization::<Self>::default())
            .default_clone_behavior()
    }
}

impl<T, A> Relationship for ContainedBy<T, A>
where
    T: 'static + Send + Sync,
    A: Arity,
{
    type RelationshipTarget = Contains<T, A>;

    fn get(&self) -> Entity {
        self.0
    }

    fn from(entity: Entity) -> Self {
        Self(entity, PhantomData)
    }
}

impl<T, A> RelationshipTarget for Contains<T, A>
where
    T: 'static + Send + Sync,
    A: Arity,
{
    const LINKED_SPAWN: bool = true;
    type Relationship = ContainedBy<T, A>;
    type Collection = A::Collection;

    fn collection(&self) -> &Self::Collection {
        &self.0
    }

    fn collection_mut_risky(&mut self) -> &mut Self::Collection {
        &mut self.0
    }

    fn from_collection_risky(collection: Self::Collection) -> Self {
        Self(collection, PhantomData)
    }
}
