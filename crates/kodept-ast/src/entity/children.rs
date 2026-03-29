use crate::arity::{Arity, Optional, Plural, Singular};
use crate::prelude::{ASTNode, NodeId};
use crate::properties::NodeProperty;
use crate::relationship::NodeRelationship;
use crate::syntax_tree::children::{Family, HasChild};
use derive_more::{Display, Error, From};
use kodept_ecs::component::Mutable;
use kodept_ecs::entity::{Entity, EntitySetIterator};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::exported::bevy_ecs::query::QueryIter;
use kodept_ecs::query::{
    QueryData, QueryEntityError, QueryFilter, QueryItem, QueryManyIter, QueryManyUniqueIter,
    ROQueryItem, ReadOnlyQueryData,
};
use kodept_ecs::relationship::{
    Relationship, RelationshipSourceCollection, RelationshipTarget, SourceIter,
};
use kodept_ecs::system::{Query, SystemParam};
use smallvec::SmallVec;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::iter::FusedIterator;
use std::marker::PhantomData;

type Rel<T, Tag> = <T as NodeRelationship<Tag, <T as Family<Tag>>::Arity>>::Relationship;
type Target<T> = <T as Relationship>::RelationshipTarget;
type Collection<T> = <Target<T> as RelationshipTarget>::Collection;
type Container<A, T> = <A as TryFromIter>::Container<T>;

#[repr(transparent)]
pub struct Property<T: NodeProperty>(PhantomData<&'static T>);
#[repr(transparent)]
pub struct MutProperty<T: NodeProperty<Mutability = Mutable>>(PhantomData<&'static mut T>);

pub trait NodeQueryData<Of>: QueryData {}

mod node_query_data_impls {
    use crate::entity::children::{MutProperty, Property};
    use crate::node_id::NodeId;
    use crate::prelude::NodeQueryData;
    use crate::properties::{HasProperty, NodeProperty, RequireProperty};
    use crate::traits::ASTNode;
    use kodept_ecs::archetype::Archetype;
    use kodept_ecs::change_detection::Tick;
    use kodept_ecs::component::{Component, ComponentId, Components, Mutable};
    use kodept_ecs::entity::Entity;
    use kodept_ecs::query::{
        Access, ArchetypeQueryData, EcsAccessType, FilteredAccess, QueryData, ReadOnlyQueryData,
        WorldQuery,
    };
    use kodept_ecs::storage::{Table, TableRow};
    use kodept_ecs::world::{UnsafeWorldCell, World};

    impl<T> NodeQueryData<T> for () {}
    impl<A, T> NodeQueryData<T> for (A,) where A: NodeQueryData<T> {}
    impl<A, B, T> NodeQueryData<T> for (A, B)
    where
        A: NodeQueryData<T>,
        B: NodeQueryData<T>,
    {
    }
    impl<A, B, C, T> NodeQueryData<T> for (A, B, C)
    where
        A: NodeQueryData<T>,
        B: NodeQueryData<T>,
        C: NodeQueryData<T>,
    {
    }
    impl<A, B, C, D, T> NodeQueryData<T> for (A, B, C, D)
    where
        A: NodeQueryData<T>,
        B: NodeQueryData<T>,
        C: NodeQueryData<T>,
        D: NodeQueryData<T>,
    {
    }
    impl<A, B, C, D, E, T> NodeQueryData<T> for (A, B, C, D, E)
    where
        A: NodeQueryData<T>,
        B: NodeQueryData<T>,
        C: NodeQueryData<T>,
        D: NodeQueryData<T>,
        E: NodeQueryData<T>,
    {
    }
    impl<T: ASTNode> NodeQueryData<T> for NodeId<T> {}
    impl<T> NodeQueryData<T> for &Archetype {}
    impl<'a, T: ASTNode> NodeQueryData<T> for &'a T {}
    impl<'a, T> NodeQueryData<T> for &'a mut T where T: ASTNode<Mutability = Mutable> {}
    impl<P, T> NodeQueryData<T> for Option<Property<P>>
    where
        P: NodeProperty + Component,
        T: HasProperty<P>,
    {
    }
    impl<P, T> NodeQueryData<T> for Property<P>
    where
        P: NodeProperty + Component,
        T: RequireProperty<P>,
    {
    }
    impl<P, T> NodeQueryData<T> for Option<MutProperty<P>>
    where
        P: NodeProperty<Mutability = Mutable>,
        T: HasProperty<P>,
    {
    }
    impl<P, T> NodeQueryData<T> for MutProperty<P>
    where
        P: NodeProperty<Mutability = Mutable>,
        T: RequireProperty<P>,
    {
    }

    #[allow(unsafe_code)]
    unsafe impl<T: NodeProperty> WorldQuery for Property<T> {
        type Fetch<'w> = <&'static T as WorldQuery>::Fetch<'w>;
        type State = <&'static T as WorldQuery>::State;

        fn shrink_fetch<'wlong: 'wshort, 'wshort>(
            fetch: Self::Fetch<'wlong>,
        ) -> Self::Fetch<'wshort> {
            <&'static T as WorldQuery>::shrink_fetch(fetch)
        }

        unsafe fn init_fetch<'w, 's>(
            world: UnsafeWorldCell<'w>,
            state: &'s Self::State,
            last_run: Tick,
            this_run: Tick,
        ) -> Self::Fetch<'w> {
            unsafe { <&'static T as WorldQuery>::init_fetch(world, state, last_run, this_run) }
        }

        const IS_DENSE: bool = <&'static T as WorldQuery>::IS_DENSE;

        unsafe fn set_archetype<'w, 's>(
            fetch: &mut Self::Fetch<'w>,
            state: &'s Self::State,
            archetype: &'w Archetype,
            table: &'w Table,
        ) {
            unsafe { <&'static T as WorldQuery>::set_archetype(fetch, state, archetype, table) }
        }

        unsafe fn set_table<'w, 's>(
            fetch: &mut Self::Fetch<'w>,
            state: &'s Self::State,
            table: &'w Table,
        ) {
            unsafe { <&'static T as WorldQuery>::set_table(fetch, state, table) }
        }

        fn update_component_access(state: &Self::State, access: &mut FilteredAccess) {
            <&'static T as WorldQuery>::update_component_access(state, access)
        }

        fn init_state(world: &mut World) -> Self::State {
            <&'static T as WorldQuery>::init_state(world)
        }

        fn get_state(components: &Components) -> Option<Self::State> {
            <&'static T as WorldQuery>::get_state(components)
        }

        fn matches_component_set(
            state: &Self::State,
            set_contains_id: &impl Fn(ComponentId) -> bool,
        ) -> bool {
            <&'static T as WorldQuery>::matches_component_set(state, set_contains_id)
        }
    }

    #[allow(unsafe_code)]
    unsafe impl<T: NodeProperty> QueryData for Property<T> {
        const IS_READ_ONLY: bool = <&'static T as QueryData>::IS_READ_ONLY;
        const IS_ARCHETYPAL: bool = <&'static T as QueryData>::IS_ARCHETYPAL;
        type ReadOnly = Self;
        type Item<'w, 's> = <&'static T as QueryData>::Item<'w, 's>;

        fn shrink<'wlong: 'wshort, 'wshort, 's>(
            item: Self::Item<'wlong, 's>,
        ) -> Self::Item<'wshort, 's> {
            <&'static T as QueryData>::shrink(item)
        }

        fn provide_extra_access(
            _state: &mut Self::State,
            _access: &mut Access,
            _available_access: &Access,
        ) {
            <&'static T as QueryData>::provide_extra_access(_state, _access, _available_access)
        }

        unsafe fn fetch<'w, 's>(
            state: &'s Self::State,
            fetch: &mut Self::Fetch<'w>,
            entity: Entity,
            table_row: TableRow,
        ) -> Option<Self::Item<'w, 's>> {
            unsafe { <&'static T as QueryData>::fetch(state, fetch, entity, table_row) }
        }

        fn iter_access(state: &Self::State) -> impl Iterator<Item = EcsAccessType<'_>> {
            <&'static T as QueryData>::iter_access(state)
        }
    }

    #[allow(unsafe_code)]
    unsafe impl<T: NodeProperty> ReadOnlyQueryData for Property<T> {}
    impl<T: NodeProperty> ArchetypeQueryData for Property<T> {}

    #[allow(unsafe_code)]
    unsafe impl<T: NodeProperty<Mutability = Mutable>> WorldQuery for MutProperty<T> {
        type Fetch<'w> = <&'static mut T as WorldQuery>::Fetch<'w>;
        type State = <&'static mut T as WorldQuery>::State;

        fn shrink_fetch<'wlong: 'wshort, 'wshort>(
            fetch: Self::Fetch<'wlong>,
        ) -> Self::Fetch<'wshort> {
            <&'static mut T as WorldQuery>::shrink_fetch(fetch)
        }

        unsafe fn init_fetch<'w, 's>(
            world: UnsafeWorldCell<'w>,
            state: &'s Self::State,
            last_run: Tick,
            this_run: Tick,
        ) -> Self::Fetch<'w> {
            unsafe { <&'static mut T as WorldQuery>::init_fetch(world, state, last_run, this_run) }
        }

        const IS_DENSE: bool = <&'static mut T as WorldQuery>::IS_DENSE;

        unsafe fn set_archetype<'w, 's>(
            fetch: &mut Self::Fetch<'w>,
            state: &'s Self::State,
            archetype: &'w Archetype,
            table: &'w Table,
        ) {
            unsafe { <&'static mut T as WorldQuery>::set_archetype(fetch, state, archetype, table) }
        }

        unsafe fn set_table<'w, 's>(
            fetch: &mut Self::Fetch<'w>,
            state: &'s Self::State,
            table: &'w Table,
        ) {
            unsafe { <&'static mut T as WorldQuery>::set_table(fetch, state, table) }
        }

        fn update_component_access(state: &Self::State, access: &mut FilteredAccess) {
            <&'static mut T as WorldQuery>::update_component_access(state, access)
        }

        fn init_state(world: &mut World) -> Self::State {
            <&'static mut T as WorldQuery>::init_state(world)
        }

        fn get_state(components: &Components) -> Option<Self::State> {
            <&'static mut T as WorldQuery>::get_state(components)
        }

        fn matches_component_set(
            state: &Self::State,
            set_contains_id: &impl Fn(ComponentId) -> bool,
        ) -> bool {
            <&'static mut T as WorldQuery>::matches_component_set(state, set_contains_id)
        }
    }

    #[allow(unsafe_code)]
    unsafe impl<T: NodeProperty<Mutability = Mutable>> QueryData for MutProperty<T> {
        const IS_READ_ONLY: bool = <&'static mut T as QueryData>::IS_READ_ONLY;
        const IS_ARCHETYPAL: bool = <&'static mut T as QueryData>::IS_ARCHETYPAL;
        type ReadOnly = Property<T>;
        type Item<'w, 's> = <&'static mut T as QueryData>::Item<'w, 's>;

        fn shrink<'wlong: 'wshort, 'wshort, 's>(
            item: Self::Item<'wlong, 's>,
        ) -> Self::Item<'wshort, 's> {
            <&'static mut T as QueryData>::shrink(item)
        }

        fn provide_extra_access(
            _state: &mut Self::State,
            _access: &mut Access,
            _available_access: &Access,
        ) {
            <&'static mut T as QueryData>::provide_extra_access(_state, _access, _available_access)
        }

        unsafe fn fetch<'w, 's>(
            state: &'s Self::State,
            fetch: &mut Self::Fetch<'w>,
            entity: Entity,
            table_row: TableRow,
        ) -> Option<Self::Item<'w, 's>> {
            unsafe { <&'static mut T as QueryData>::fetch(state, fetch, entity, table_row) }
        }

        fn iter_access(state: &Self::State) -> impl Iterator<Item = EcsAccessType<'_>> {
            <&'static mut T as QueryData>::iter_access(state)
        }
    }

    impl<T: NodeProperty<Mutability = Mutable>> ArchetypeQueryData for MutProperty<T> {}
}

pub struct ChildrenFetch<'w, 's, Data, Parent, Tag = (), Id = NodeId, Filter = ()>
where
    Data: QueryData,
    Filter: QueryFilter,
    Parent: Family<Tag>,
    Id: ReadOnlyQueryData,
{
    query: Query<'w, 's, (Id, Data, &'static Rel<Parent, Tag>), Filter>,
    collection: Option<&'w Target<Rel<Parent, Tag>>>,
}

pub struct ChildrenIter<'w, 's, Data, R, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    R: Relationship,
    Id: ReadOnlyQueryData,
{
    inner: Option<
        QueryManyIter<
            'w,
            's,
            (Id, Data, &'static R),
            Filter,
            SourceIter<'w, R::RelationshipTarget>,
        >,
    >,
}

pub struct ChildrenMutIter<'w, 's, Data, R, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    R: Relationship,
    Id: ReadOnlyQueryData,
    SourceIter<'w, R::RelationshipTarget>: EntitySetIterator,
{
    inner: Option<
        QueryManyUniqueIter<
            'w,
            's,
            (Id, Data, &'static R),
            Filter,
            SourceIter<'w, R::RelationshipTarget>,
        >,
    >,
}

#[derive(SystemParam)]
pub struct HierarchicalQuery<
    'w,
    's,
    Parent,
    Tag = (),
    ParentData = &'static Parent,
    ChildData = (),
    Filter = (),
> where
    Parent: Family<Tag>,
    Parent: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: NodeQueryData<Parent> + 'static,
    ChildData: QueryData + 'static,
{
    parent_query: Query<
        'w,
        's,
        (
            NodeId<Parent>,
            ParentData,
            Option<&'static Target<Rel<Parent, Tag>>>,
        ),
        Filter,
    >,
    children_query: Query<'w, 's, (NodeId, ChildData, &'static Rel<Parent, Tag>)>,
}

#[derive(SystemParam)]
pub struct NarrowHierarchicalQuery<
    'w,
    's,
    Parent,
    Child,
    Tag = (),
    ParentData = &'static Parent,
    ChildData = &'static Child,
    Filter = (),
> where
    Parent: HasChild<Child, Tag>,
    Parent: ASTNode,
    Child: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: NodeQueryData<Parent> + 'static,
    ChildData: NodeQueryData<Child> + 'static,
{
    parent_query: Query<
        'w,
        's,
        (
            NodeId<Parent>,
            ParentData,
            Option<&'static Target<Rel<Parent, Tag>>>,
        ),
        Filter,
    >,
    children_query: Query<'w, 's, (NodeId<Child>, ChildData, &'static Rel<Parent, Tag>)>,
}

pub trait TryFromIter {
    type Container<T>;
    type Error: Error;

    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error>;
}

#[derive(Debug, Display, Error)]
pub enum SingleChildError {
    #[display("Expected exactly one child, found zero")]
    Zero,
    #[display("Expected exactly one child, found {_0}")]
    AtLeastTwo(#[error(not(source))] usize),
}

#[derive(Debug, Display, Error)]
pub enum OptionChildError {
    #[display("Expected at most one child, found {_0}")]
    AtLeastTwo(#[error(not(source))] usize),
}

#[derive(From)]
pub enum HierarchicalError<E> {
    #[from(ignore)]
    WrongContainerSize(E),
    CannotQuery(QueryEntityError),
}

impl<E: Debug> Debug for HierarchicalError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HierarchicalError::WrongContainerSize(x) => f
                .debug_tuple("HierarchicalError::WrongContainerSize")
                .field(x)
                .finish(),
            HierarchicalError::CannotQuery(x) => f
                .debug_tuple("HierarchicalError::CannotQuery")
                .field(x)
                .finish(),
        }
    }
}

impl<E: Display> Display for HierarchicalError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HierarchicalError::WrongContainerSize(x) => {
                write!(f, "Wrong container size: {x}")
            }
            HierarchicalError::CannotQuery(x) => {
                write!(f, "{x}")
            }
        }
    }
}

impl<E: std::error::Error> std::error::Error for HierarchicalError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HierarchicalError::WrongContainerSize(_) => None,
            HierarchicalError::CannotQuery(x) => Some(x),
        }
    }
}

pub struct LayerFetch<'w, 's, Parent, Tag, ParentData, ChildData, Filter>
where
    Parent: Family<Tag>,
    Parent: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: NodeQueryData<Parent> + 'static,
    ChildData: QueryData + 'static,
{
    parent_iter: QueryIter<
        'w,
        's,
        (
            NodeId<Parent>,
            ParentData,
            Option<&'static Target<Rel<Parent, Tag>>>,
        ),
        Filter,
    >,
    children: Query<'w, 's, (NodeId, ChildData, &'static Rel<Parent, Tag>)>,
}

impl<'w, 's, Parent, Tag, ParentData, ChildData, Filter>
    LayerFetch<'w, 's, Parent, Tag, ParentData, ChildData, Filter>
where
    Parent: Family<Tag>,
    Parent: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: NodeQueryData<Parent> + 'static,
    ChildData: QueryData + 'static,
{
    pub fn fetch_next(
        &mut self,
    ) -> Option<(
        NodeId<Parent>,
        QueryItem<'_, 's, ParentData>,
        ChildrenFetch<'_, 's, ChildData, Parent, Tag, NodeId, ()>,
    )> {
        let (parent_id, parent_data, children) = self.parent_iter.next()?;
        let children_fetch = ChildrenFetch {
            collection: children,
            query: self.children.reborrow()
        };
        Some((parent_id, ParentData::shrink(parent_data), children_fetch))
    }
}

impl<'w, 's, T, Tag, ParentData, ChildData, Filter>
    HierarchicalQuery<'w, 's, T, Tag, ParentData, ChildData, Filter>
where
    T: Family<Tag>,
    T: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: NodeQueryData<T> + 'static,
    ChildData: QueryData + 'static,
{
    pub fn iter_by_layers(
        &mut self,
    ) -> impl Iterator<
        Item = (
            NodeId<T>,
            QueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag>,
        ),
    > {
        self.parent_query
            .iter_mut()
            .map(|(parent_id, parent_data, children)| {
                (
                    parent_id,
                    parent_data,
                    ChildrenFetch {
                        collection: children,
                        query: self.children_query.as_readonly(),
                    },
                )
            })
    }

    pub fn iter_mut_by_layers(
        &mut self,
    ) -> LayerFetch<'_, 's, T, Tag, ParentData, ChildData, Filter> {
        LayerFetch {
            parent_iter: self.parent_query.iter_mut(),
            children: self.children_query.reborrow()
        }
    }

    pub fn par_iter_by_layers(
        &mut self,
        f: impl for<'ww> Fn(
            NodeId<T>,
            QueryItem<'ww, 's, ParentData>,
            ChildrenFetch<'ww, 's, ChildData::ReadOnly, T, Tag>,
        ) + Send
        + Sync
        + Clone,
    ) {
        self.parent_query
            .par_iter_mut()
            .for_each(|(parent_id, parent_data, children)| {
                f(
                    parent_id,
                    parent_data,
                    ChildrenFetch {
                        collection: children,
                        query: self.children_query.as_readonly(),
                    },
                )
            });
    }

    pub fn get_down_mut(
        &mut self,
        id: NodeId<T>,
    ) -> (
        QueryItem<'_, 's, ParentData>,
        ChildrenFetch<'_, 's, ChildData, T, Tag>,
    ) {
        self.try_get_down_mut(id).unwrap()
    }

    pub fn try_get_down_mut(
        &mut self,
        id: NodeId<T>,
    ) -> Result<
        (
            QueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData, T, Tag>,
        ),
        HierarchicalError<Infallible>,
    > {
        let (_, parent, children) = self.parent_query.get_mut(id.entity())?;
        Ok((
            parent,
            ChildrenFetch {
                collection: children,
                query: self.children_query.reborrow(),
            },
        ))
    }

    // Retrieves from world references to both parent and child by given *parent* id.
    pub fn get_down(
        &self,
        id: NodeId<T>,
    ) -> (
        ROQueryItem<'_, 's, ParentData>,
        ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag>,
    ) {
        self.try_get_down(id)
            .expect("Cannot collect children into container")
    }

    pub fn get_children(&self, id: NodeId<T>) -> ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag>
    where
        T::Arity: TryFromIter,
    {
        self.get_down(id).1
    }

    pub fn get_children_mut(&mut self, id: NodeId<T>) -> ChildrenFetch<'_, 's, ChildData, T, Tag>
    where
        T::Arity: TryFromIter,
    {
        self.get_down_mut(id).1
    }

    pub fn try_get_down(
        &self,
        id: NodeId<T>,
    ) -> Result<
        (
            ROQueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag>,
        ),
        HierarchicalError<Infallible>,
    > {
        let (_, parent, children) = self.parent_query.get(id.entity())?;
        Ok((
            parent,
            ChildrenFetch {
                collection: children,
                query: self.children_query.as_readonly(),
            },
        ))
    }
}

impl<'w, 's, Data, T, Tag, Id, Filter> IntoIterator
    for ChildrenFetch<'w, 's, Data, T, Tag, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: ReadOnlyQueryData,
{
    type Item = (ROQueryItem<'w, 's, Id>, QueryItem<'w, 's, Data>);
    type IntoIter = ChildrenMutIter<'w, 's, Data, Rel<T, Tag>, Id, Filter>;

    fn into_iter(self) -> Self::IntoIter {
        let query = self.query;

        ChildrenMutIter {
            inner: self
                .collection
                .map(|it| query.iter_many_unique_inner(it.iter())),
        }
    }
}

impl<'ww, 'w, 's, Data, T, Tag, Id, Filter> IntoIterator
    for &'ww ChildrenFetch<'w, 's, Data, T, Tag, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: ReadOnlyQueryData,
{
    type Item = (ROQueryItem<'ww, 's, Id>, ROQueryItem<'ww, 's, Data>);
    type IntoIter = ChildrenIter<'ww, 's, Data::ReadOnly, Rel<T, Tag>, Id, Filter>;

    fn into_iter(self) -> Self::IntoIter {
        let query = self.query.as_readonly();
        ChildrenIter {
            inner: self.collection.map(|it| query.iter_many_inner(it.iter())),
        }
    }
}

impl<'w, 's, Data, Rel, Id, Filter> Iterator for ChildrenMutIter<'w, 's, Data, Rel, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    Rel: Relationship,
    Id: ReadOnlyQueryData,
    SourceIter<'w, Rel::RelationshipTarget>: EntitySetIterator,
{
    type Item = (ROQueryItem<'w, 's, Id>, QueryItem<'w, 's, Data>);

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.inner.as_mut()?;
        let value = iter.next()?;
        Some((value.0, value.1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            None => (0, Some(0)),
            Some(inner) => inner.size_hint(),
        }
    }
}

impl<'w, 's, Data, Rel, Id, Filter> FusedIterator for ChildrenMutIter<'w, 's, Data, Rel, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    Rel: Relationship,
    Id: ReadOnlyQueryData,
    SourceIter<'w, Rel::RelationshipTarget>: EntitySetIterator,
{
}

impl<'w, 's, Data, Rel, Id, Filter> Iterator for ChildrenIter<'w, 's, Data, Rel, Id, Filter>
where
    Data: ReadOnlyQueryData,
    Filter: QueryFilter,
    Rel: Relationship,
    Id: ReadOnlyQueryData,
{
    type Item = (ROQueryItem<'w, 's, Id>, ROQueryItem<'w, 's, Data>);

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.inner.as_mut()?;
        let value = iter.next()?;
        Some((value.0, value.1))
    }
}

impl<'w, 's, Data, Rel, Id, Filter> FusedIterator for ChildrenIter<'w, 's, Data, Rel, Id, Filter>
where
    Data: ReadOnlyQueryData,
    Filter: QueryFilter,
    Rel: Relationship,
    Id: ReadOnlyQueryData,
{
}

impl<'w, 's, Data, Rel, Id, Filter> DoubleEndedIterator
    for ChildrenIter<'w, 's, Data, Rel, Id, Filter>
where
    Data: ReadOnlyQueryData,
    Filter: QueryFilter,
    Rel: Relationship,
    Id: ReadOnlyQueryData,
    SourceIter<'w, Rel::RelationshipTarget>: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let iter = self.inner.as_mut()?;
        let value = iter.next_back()?;
        Some((value.0, value.1))
    }
}

impl<'w, 's, Data, T, Tag, Id, Filter> ChildrenFetch<'w, 's, Data, T, Tag, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: ReadOnlyQueryData,
{
    #[inline]
    pub fn iter(&self) -> ChildrenIter<'_, 's, Data::ReadOnly, Rel<T, Tag>, Id, Filter> {
        self.into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ChildrenMutIter<'_, 's, Data, Rel<T, Tag>, Id, Filter> {
        ChildrenMutIter {
            inner: self
                .collection
                .map(|it| self.query.iter_many_unique_mut(it.iter())),
        }
    }

    #[inline]
    #[track_caller]
    pub fn collect(self) -> Container<T::Arity, (QueryItem<'w, 's, Id>, QueryItem<'w, 's, Data>)>
    where
        T::Arity: TryFromIter,
    {
        self.try_collect().unwrap()
    }

    pub fn try_collect(
        self,
    ) -> Result<
        Container<T::Arity, (QueryItem<'w, 's, Id>, QueryItem<'w, 's, Data>)>,
        HierarchicalError<<T::Arity as TryFromIter>::Error>,
    >
    where
        T::Arity: TryFromIter,
    {
        let Some(children) = self.collection else {
            let container = <T::Arity as TryFromIter>::try_from_iter(None);
            return container.map_err(|e| HierarchicalError::WrongContainerSize(e));
        };
        let collection: &<T::Arity as Arity>::Collection = children.collection();
        let iter = self
            .query
            .iter_many_unique_inner(collection.iter())
            .map(|it| (it.0, it.1));

        <T::Arity as TryFromIter>::try_from_iter(iter)
            .map_err(|e| HierarchicalError::WrongContainerSize(e))
    }

    pub fn as_slice(&self) -> &[Entity]
    where
        Collection<Rel<T, Tag>>: AsRef<[Entity]>,
    {
        match self.collection {
            None => &[],
            Some(x) => {
                let collection = x.collection();
                let slice = collection.as_ref();
                slice
            }
        }
    }
}

impl<'w, 's, T, U, Tag, ParentData, ChildData, Filter>
    NarrowHierarchicalQuery<'w, 's, T, U, Tag, ParentData, ChildData, Filter>
where
    T: HasChild<U, Tag>,
    T: ASTNode,
    U: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: NodeQueryData<T> + 'static,
    ChildData: NodeQueryData<U> + 'static,
{
    pub fn iter_by_layers(
        &mut self,
    ) -> impl Iterator<
        Item = (
            NodeId<T>,
            ParentData::Item<'_, 's>,
            ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>,
        ),
    > {
        self.parent_query
            .iter_mut()
            .map(|(parent_id, parent_data, children)| {
                (
                    parent_id,
                    parent_data,
                    ChildrenFetch {
                        query: self.children_query.as_readonly(),
                        collection: children,
                    },
                )
            })
    }

    pub fn par_iter_by_layers(
        &mut self,
        callback: impl for<'ww> Fn(
            NodeId<T>,
            ParentData::Item<'ww, 's>,
            ChildrenFetch<'ww, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>,
        ) + Send
        + Sync
        + Clone,
    ) {
        self.parent_query
            .par_iter_mut()
            .for_each(|(parent_id, parent_data, children)| {
                callback(
                    parent_id,
                    parent_data,
                    ChildrenFetch {
                        query: self.children_query.as_readonly(),
                        collection: children,
                    },
                )
            })
    }

    /// Retrieves from world references to both parent and child by given *parent* id.
    pub fn get_down(
        &self,
        id: NodeId<T>,
    ) -> (
        ROQueryItem<'_, 's, ParentData>,
        ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>,
    ) {
        self.try_get_down(id)
            .expect("Cannot collect children into container")
    }

    #[inline]
    pub fn get_children(
        &self,
        id: NodeId<T>,
    ) -> ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>> {
        self.get_down(id).1
    }

    #[inline]
    pub fn get_children_mut(
        &mut self,
        id: NodeId<T>,
    ) -> ChildrenFetch<'_, 's, ChildData, T, Tag, NodeId<U>> {
        self.get_down_mut(id).1
    }

    pub fn try_get_down(
        &self,
        id: NodeId<T>,
    ) -> Result<
        (
            ROQueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>,
        ),
        HierarchicalError<Infallible>,
    > {
        let (_, parent, children) = self.parent_query.get(id.entity())?;
        Ok((
            parent,
            ChildrenFetch {
                collection: children,
                query: self.children_query.as_readonly(),
            },
        ))
    }

    pub fn get_down_mut(
        &mut self,
        id: NodeId<T>,
    ) -> (
        QueryItem<'_, 's, ParentData>,
        ChildrenFetch<'_, 's, ChildData, T, Tag, NodeId<U>>,
    ) {
        self.try_get_down_mut(id).unwrap()
    }

    pub fn try_get_down_mut(
        &mut self,
        id: NodeId<T>,
    ) -> Result<
        (
            QueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData, T, Tag, NodeId<U>>,
        ),
        HierarchicalError<Infallible>,
    > {
        let (_, parent, children) = self.parent_query.get_mut(id.entity())?;
        Ok((
            parent,
            ChildrenFetch {
                collection: children,
                query: self.children_query.reborrow(),
            },
        ))
    }

    /// Retrieves from world references to both parent and child by given *child* id.
    /// Essentially, this method costs two constant lookups into world, so overall time complexity is `O(1)`
    ///
    /// # Errors
    ///
    /// This function will return an error if there is no such child with given id or there is no parent of type [`T`]
    pub fn get_up(
        &self,
        id: NodeId<U>,
    ) -> Result<
        (
            NodeId<T>,
            ROQueryItem<'_, 's, ParentData>,
            ROQueryItem<'_, 's, ChildData>,
        ),
        QueryEntityError,
    > {
        let (_, child, parent) = self.children_query.get(id.entity())?;
        let (parent_id, parent, _) = self.parent_query.get(parent.get())?;
        Ok((parent_id.into(), parent, child))
    }
}

impl TryFromIter for Singular {
    type Container<T> = T;
    type Error = SingleChildError;

    #[inline]
    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error> {
        Optional::try_from_iter(iter)
            .map_err(|OptionChildError::AtLeastTwo(n)| SingleChildError::AtLeastTwo(n))
            .and_then(|it| it.ok_or(SingleChildError::Zero))
    }
}

impl TryFromIter for Optional {
    type Container<T> = Option<T>;

    type Error = OptionChildError;

    #[inline]
    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error> {
        let mut iter = iter.into_iter().fuse();
        let first = iter.next();
        let second = iter.next();
        match (first, second) {
            (None, None) => Ok(None),
            (Some(x), None) => Ok(Some(x)),
            (None, Some(_)) => unreachable!("Iterator is not fused"),
            (Some(_), Some(_)) => Err(OptionChildError::AtLeastTwo(iter.count() + 2)),
        }
    }
}

impl TryFromIter for Plural {
    type Container<T> = SmallVec<[T; 2]>;

    type Error = Infallible;

    #[inline]
    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error> {
        Ok(SmallVec::from_iter(iter))
    }
}
