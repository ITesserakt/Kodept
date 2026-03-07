use crate::properties::Node;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::change_detection::Tick;
use kodept_ecs::component::{ComponentId, Components};
use kodept_ecs::entity::{ContainsEntity, Entity, EntityEquivalent, EntityMapper, MapEntities};
use kodept_ecs::query::{
    Access, ArchetypeQueryData, EcsAccessType, FilteredAccess, QueryData, ReadOnlyQueryData,
    ReleaseStateQueryData, With, WorldQuery,
};
use kodept_ecs::relationship::RelationshipSourceCollection;
use kodept_ecs::storage::{Table, TableRow};
use kodept_ecs::world::{UnsafeWorldCell, World};
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Ids that their associated type can be erased
pub trait Erase<Erased = NodeId> {
    fn erase(self) -> Erased;
}

impl Erase<NodeId> for Entity {
    #[inline(always)]
    fn erase(self) -> NodeId {
        self.into()
    }
}

impl<T> Erase<NodeId> for NodeId<T> {
    #[inline(always)]
    fn erase(self) -> NodeId {
        self.cast()
    }
}

impl<T: Erase> Erase<Entity> for T {
    #[inline(always)]
    fn erase(self) -> Entity {
        T::erase(self).entity
    }
}

pub struct NodeId<T = ()> {
    entity: Entity,
    _phantom: PhantomData<T>,
}

impl<T> From<Entity> for NodeId<T> {
    #[inline(always)]
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

    #[inline(always)]
    pub fn cast<U>(self) -> NodeId<U> {
        NodeId {
            entity: self.entity,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

impl<T> Clone for NodeId<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeId<T> {}

impl<T> Deref for NodeId<T> {
    type Target = Entity;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T> DerefMut for NodeId<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<T> Debug for NodeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.entity, f)
    }
}

impl<T> PartialEq for NodeId<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

impl<T> Eq for NodeId<T> {}

impl<T> PartialOrd for NodeId<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.entity.partial_cmp(&other.entity)
    }
}

impl<T> Ord for NodeId<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.entity.cmp(&other.entity)
    }
}

impl<T> Hash for NodeId<T> {
    #[inline(always)]
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
    #[inline(always)]
    fn borrow(&self) -> &Entity {
        &self.entity
    }
}

impl<T> RelationshipSourceCollection for NodeId<T> {
    type SourceIter<'a>
        = std::option::IntoIter<Entity>
    where
        T: 'a;

    #[inline(always)]
    fn new() -> Self {
        Self::NULL
    }

    #[inline(always)]
    fn with_capacity(_capacity: usize) -> Self {
        Self::NULL
    }

    #[inline(always)]
    fn reserve(&mut self, additional: usize) {
        self.entity.reserve(additional)
    }

    #[inline(always)]
    fn add(&mut self, entity: Entity) -> bool {
        self.entity.add(entity)
    }

    #[inline(always)]
    fn remove(&mut self, entity: Entity) -> bool {
        self.entity.remove(entity)
    }

    #[inline(always)]
    fn iter(&self) -> Self::SourceIter<'_> {
        self.entity.iter()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.entity.len()
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.entity.clear()
    }

    #[inline(always)]
    fn shrink_to_fit(&mut self) {
        self.entity.shrink_to_fit()
    }

    #[inline(always)]
    fn extend_from_iter(&mut self, entities: impl IntoIterator<Item = Entity>) {
        self.entity.extend_from_iter(entities);
    }
}

impl<T> ContainsEntity for NodeId<T> {
    #[inline(always)]
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[allow(unsafe_code)]
unsafe impl<T> EntityEquivalent for NodeId<T> {}

impl<T> MapEntities for NodeId<T> {
    #[inline(always)]
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.entity.map_entities(entity_mapper)
    }
}

impl<T> From<NodeId<T>> for Entity {
    #[inline(always)]
    fn from(value: NodeId<T>) -> Self {
        value.entity
    }
}

// SAFETY: NodeId is just a thin wrapper around Entity.
//         Queries with NodeId behaves as entity with `With<T>` constrain
#[allow(unsafe_code)]
unsafe impl<T> WorldQuery for NodeId<T>
where
    T: crate::prelude::ASTNode,
{
    type Fetch<'w> = (
        <Entity as WorldQuery>::Fetch<'w>,
        <With<T> as WorldQuery>::Fetch<'w>,
    );
    type State = (
        <Entity as WorldQuery>::State,
        <With<T> as WorldQuery>::State,
    );

    #[inline]
    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        (
            Entity::shrink_fetch(fetch.0),
            With::<T>::shrink_fetch(fetch.1),
        )
    }

    #[inline]
    unsafe fn init_fetch<'w, 's>(
        world: UnsafeWorldCell<'w>,
        state: &'s Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        (
            unsafe { Entity::init_fetch(world, &state.0, last_run, this_run) },
            unsafe { With::<T>::init_fetch(world, &state.1, last_run, this_run) },
        )
    }

    const IS_DENSE: bool = <Entity as WorldQuery>::IS_DENSE && <With<T> as WorldQuery>::IS_DENSE;

    #[inline]
    unsafe fn set_archetype<'w, 's>(
        fetch: &mut Self::Fetch<'w>,
        state: &'s Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        unsafe { Entity::set_archetype(&mut fetch.0, &state.0, archetype, table) };
        unsafe { With::<T>::set_archetype(&mut fetch.1, &state.1, archetype, table) };
    }

    #[inline]
    unsafe fn set_table<'w, 's>(
        fetch: &mut Self::Fetch<'w>,
        state: &'s Self::State,
        table: &'w Table,
    ) {
        unsafe { Entity::set_table(&mut fetch.0, &state.0, table) };
        unsafe { With::<T>::set_table(&mut fetch.1, &state.1, table) };
    }

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut FilteredAccess) {
        Entity::update_component_access(&state.0, access);
        With::<T>::update_component_access(&state.1, access);
    }

    #[inline]
    fn init_state(world: &mut World) -> Self::State {
        (Entity::init_state(world), With::<T>::init_state(world))
    }

    #[inline]
    fn get_state(components: &Components) -> Option<Self::State> {
        Some((
            Entity::get_state(components)?,
            With::<T>::get_state(components)?,
        ))
    }

    #[inline]
    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        Entity::matches_component_set(&state.0, set_contains_id)
            && With::<T>::matches_component_set(&state.1, set_contains_id)
    }
}

// SAFETY: NodeId is just a thin wrapper around Entity.
//         Queries with NodeId<T> behaves as entity with `With<T>` constrain
#[allow(unsafe_code)]
unsafe impl<T> QueryData for NodeId<T>
where
    T: crate::prelude::ASTNode,
{
    const IS_READ_ONLY: bool = <Entity as QueryData>::IS_READ_ONLY;
    const IS_ARCHETYPAL: bool = <Entity as QueryData>::IS_ARCHETYPAL;
    type ReadOnly = Self;
    type Item<'w, 's> = NodeId<T>;

    #[inline]
    fn shrink<'wlong: 'wshort, 'wshort, 's>(
        item: Self::Item<'wlong, 's>,
    ) -> Self::Item<'wshort, 's> {
        item
    }

    #[inline]
    fn provide_extra_access(
        _state: &mut Self::State,
        _access: &mut Access,
        _available_access: &Access,
    ) {
        Entity::provide_extra_access(&mut _state.0, _access, _available_access);
    }

    #[inline]
    unsafe fn fetch<'w, 's>(
        _: &'s Self::State,
        _: &mut Self::Fetch<'w>,
        entity: Entity,
        _: TableRow,
    ) -> Option<Self::Item<'w, 's>> {
        Some(NodeId {
            entity,
            _phantom: PhantomData,
        })
    }

    #[inline]
    fn iter_access(state: &Self::State) -> impl Iterator<Item = EcsAccessType<'_>> {
        Entity::iter_access(&state.0)
    }
}

// SAFETY: NodeId is just a thin wrapper around Entity.
//         Queries with NodeId<T> behaves as entity with `With<T>` constrain
#[allow(unsafe_code)]
unsafe impl<T> ReadOnlyQueryData for NodeId<T> where T: crate::prelude::ASTNode {}

impl<T> ReleaseStateQueryData for NodeId<T>
where
    T: crate::prelude::ASTNode,
    Entity: ReleaseStateQueryData,
{
    #[inline]
    fn release_state<'w>(item: Self::Item<'w, '_>) -> Self::Item<'w, 'static> {
        item
    }
}

impl<T> ArchetypeQueryData for NodeId<T>
where
    T: crate::prelude::ASTNode,
    Entity: ArchetypeQueryData,
{
}

// SAFETY: NodeId is just a thin wrapper around Entity.
//         Queries with NodeId behaves as entity
#[allow(unsafe_code)]
unsafe impl WorldQuery for NodeId {
    type Fetch<'w> = (
        <Entity as WorldQuery>::Fetch<'w>,
        <With<Node> as WorldQuery>::Fetch<'w>,
    );
    type State = (
        <Entity as WorldQuery>::State,
        <With<Node> as WorldQuery>::State,
    );

    #[inline]
    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        (
            Entity::shrink_fetch(fetch.0),
            With::<Node>::shrink_fetch(fetch.1),
        )
    }

    #[inline]
    unsafe fn init_fetch<'w, 's>(
        world: UnsafeWorldCell<'w>,
        state: &'s Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        (
            unsafe { Entity::init_fetch(world, &state.0, last_run, this_run) },
            unsafe { With::<Node>::init_fetch(world, &state.1, last_run, this_run) },
        )
    }

    const IS_DENSE: bool = <Entity as WorldQuery>::IS_DENSE && <With<Node> as WorldQuery>::IS_DENSE;

    #[inline]
    unsafe fn set_archetype<'w, 's>(
        fetch: &mut Self::Fetch<'w>,
        state: &'s Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        unsafe { Entity::set_archetype(&mut fetch.0, &state.0, archetype, table) };
        unsafe { With::<Node>::set_archetype(&mut fetch.1, &state.1, archetype, table) };
    }

    #[inline]
    unsafe fn set_table<'w, 's>(
        fetch: &mut Self::Fetch<'w>,
        state: &'s Self::State,
        table: &'w Table,
    ) {
        unsafe { Entity::set_table(&mut fetch.0, &state.0, table) };
        unsafe { With::<Node>::set_table(&mut fetch.1, &state.1, table) };
    }

    #[inline]
    fn update_component_access(state: &Self::State, access: &mut FilteredAccess) {
        Entity::update_component_access(&state.0, access);
        With::<Node>::update_component_access(&state.1, access);
    }

    #[inline]
    fn init_state(world: &mut World) -> Self::State {
        (Entity::init_state(world), With::<Node>::init_state(world))
    }

    #[inline]
    fn get_state(components: &Components) -> Option<Self::State> {
        Some((
            Entity::get_state(components)?,
            With::<Node>::get_state(components)?,
        ))
    }

    #[inline]
    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        Entity::matches_component_set(&state.0, set_contains_id)
            && With::<Node>::matches_component_set(&state.1, set_contains_id)
    }
}

// SAFETY: NodeId is just a thin wrapper around Entity.
//         Queries with NodeId behaves as entity
#[allow(unsafe_code)]
unsafe impl QueryData for NodeId {
    const IS_READ_ONLY: bool = <Entity as QueryData>::IS_READ_ONLY;
    const IS_ARCHETYPAL: bool = <Entity as QueryData>::IS_ARCHETYPAL;
    type ReadOnly = Self;
    type Item<'w, 's> = NodeId;

    #[inline]
    fn shrink<'wlong: 'wshort, 'wshort, 's>(
        item: Self::Item<'wlong, 's>,
    ) -> Self::Item<'wshort, 's> {
        item
    }

    #[inline]
    unsafe fn fetch<'w, 's>(
        _: &'s Self::State,
        _: &mut Self::Fetch<'w>,
        entity: Entity,
        _: TableRow,
    ) -> Option<Self::Item<'w, 's>> {
        Some(NodeId {
            entity,
            _phantom: PhantomData,
        })
    }

    #[inline]
    fn iter_access(state: &Self::State) -> impl Iterator<Item = EcsAccessType<'_>> {
        Entity::iter_access(&state.0)
    }
}

// SAFETY: NodeId is just a thin wrapper around Entity.
//         Queries with NodeId behaves as entity
#[allow(unsafe_code)]
unsafe impl ReadOnlyQueryData for NodeId {}

impl ReleaseStateQueryData for NodeId
where
    Entity: ReleaseStateQueryData,
{
    #[inline]
    fn release_state<'w>(item: Self::Item<'w, '_>) -> Self::Item<'w, 'static> {
        item
    }
}

impl ArchetypeQueryData for NodeId where Entity: ArchetypeQueryData {}
