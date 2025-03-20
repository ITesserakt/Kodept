use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use bevy_ecs::component::{Component, ComponentId, ComponentsRegistrator, Mutable, RequiredComponents, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::RelationshipTarget;
use bevy_ecs::relationship::{Relationship, RelationshipSourceCollection};
use crate::arity::Arity;
use crate::prelude::ASTNode;
use crate::syntax_tree::children::HasChild;

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

/// Describes all entities that is children for this entity
///
/// Type parameters:
/// * [T] - Associated with this relationship tag
/// * [A] - Degree of relationship
#[derive(Debug)]
pub struct Contains<T, A: Arity>(A::Collection, PhantomData<(T, A)>);

pub trait NodeRelationship<Child, Tag> {
    type Relationship: Relationship<Mutability = Mutable>;
    type RelationshipTarget: RelationshipTarget;
}

impl<T, U, Tag> NodeRelationship<U, Tag> for T
where
    T: HasChild<U, Tag> + ASTNode,
    U: ASTNode,
    Tag: 'static + Send + Sync,
{
    type Relationship = ContainedBy<Tag, T::Arity>;
    type RelationshipTarget = <Self::Relationship as Relationship>::RelationshipTarget;
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
    fn visit_entities(this: &Self, mut func: impl FnMut(Entity)) {
        use bevy_ecs::entity::VisitEntities;
        this.0.visit_entities(&mut func);
    }
    fn visit_entities_mut(this: &mut Self, mut func: impl FnMut(&mut Entity)) {
        use bevy_ecs::entity::VisitEntitiesMut;
        this.0.visit_entities_mut(&mut func);
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
    fn visit_entities(this: &Self, mut func: impl FnMut(Entity)) {
        use bevy_ecs::entity::VisitEntities;
        this.0.visit_entities(&mut func);
    }
    fn visit_entities_mut(this: &mut Self, mut func: impl FnMut(&mut Entity)) {
        use bevy_ecs::entity::VisitEntitiesMut;
        this.0.visit_entities_mut(&mut func);
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

