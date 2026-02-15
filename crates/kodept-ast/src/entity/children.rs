use crate::arity::{Optional, Plural, Singular};
use crate::prelude::{ASTNode, NodeId};
use crate::properties::{HasProperty, NodeProperty, RequireProperty};
use crate::relationship::NodeRelationship;
use crate::syntax_tree::children::{Family, HasChild};
use derive_more::{Display, Error, From};
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{Component, Mutable};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::{
    QueryData, QueryEntityError, QueryFilter, QueryItem, QueryManyIter, ROQueryItem,
    ReadOnlyQueryData,
};
use kodept_ecs::relationship::{Relationship, RelationshipSourceCollection, RelationshipTarget};
use kodept_ecs::system::{Query, SystemParam};
use kodept_ecs::world::{Mut, Ref};
use smallvec::SmallVec;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::iter::FusedIterator;

type Rel<T, Tag> = <T as NodeRelationship<Tag, <T as Family<Tag>>::Arity>>::Relationship;
type Target<T> = <T as Relationship>::RelationshipTarget;
type Container<A, T> = <A as TryFromIter>::Container<T>;

pub trait NodeQueryData<Of>: QueryData {}

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
impl<'a, T: ASTNode> NodeQueryData<T> for Ref<'a, T> {}
impl<'a, T> NodeQueryData<T> for Mut<'a, T> where T: ASTNode<Mutability = Mutable> {}
impl<P, T> NodeQueryData<T> for Option<&P>
where
    P: NodeProperty + Component,
    T: HasProperty<P>,
{
}
impl<P, T> NodeQueryData<T> for &P
where
    P: NodeProperty + Component,
    T: RequireProperty<P>,
{
}
impl<P, T> NodeQueryData<T> for Option<&mut P>
where
    P: NodeProperty<Mutability = Mutable>,
    T: HasProperty<P>,
{
}
impl<P, T> NodeQueryData<T> for &mut P
where
    P: NodeProperty<Mutability = Mutable>,
    T: RequireProperty<P>,
{
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
    inner: Option<QueryManyIter<'w, 's, (Id, Data, &'static R), Filter,
        <<R::RelationshipTarget as RelationshipTarget>::Collection as RelationshipSourceCollection>::SourceIter<'w>,
    >>,
}

#[derive(SystemParam)]
pub struct HierarchicalQuery<
    'w,
    's,
    Parent,
    Tag = (),
    ParentData = Ref<'static, Parent>,
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
    ParentData = Ref<'static, Parent>,
    ChildData = Ref<'static, Child>,
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
pub enum HierarchicalError<T: TryFromIter> {
    #[from(ignore)]
    WrongContainerSize(T::Error),
    CannotQuery(QueryEntityError),
}

impl<T: TryFromIter> Debug for HierarchicalError<T>
where
    T::Error: Debug,
{
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

impl<T: TryFromIter> Display for HierarchicalError<T>
where
    T::Error: Display,
{
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

impl<T: TryFromIter> std::error::Error for HierarchicalError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HierarchicalError::WrongContainerSize(_) => None,
            HierarchicalError::CannotQuery(x) => Some(x),
        }
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
    #[allow(unsafe_code)]
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

    pub fn par_iter_by_layers(
        &mut self,
        f: impl Fn(
            NodeId<T>,
            QueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag>,
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
}

impl<'w, 's, Data, T, Tag, Id, Filter> IntoIterator
    for ChildrenFetch<'w, 's, Data, T, Tag, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: ReadOnlyQueryData,
{
    type Item = (ROQueryItem<'w, 's, Id>, ROQueryItem<'w, 's, Data>);
    type IntoIter = ChildrenIter<'w, 's, Data::ReadOnly, Rel<T, Tag>, Id, Filter>;

    fn into_iter(self) -> Self::IntoIter {
        let query = self.query.into_readonly();

        ChildrenIter {
            inner: self.collection.map(|it| query.iter_many_inner(it.iter())),
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
    Rel: Relationship<
        RelationshipTarget: RelationshipTarget<
            Collection: RelationshipSourceCollection<SourceIter<'w>: DoubleEndedIterator>,
        >,
    >,
    Id: ReadOnlyQueryData,
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

    pub fn collect(
        self,
    ) -> Container<T::Arity, (ROQueryItem<'w, 's, Id>, ROQueryItem<'w, 's, Data>)>
    where
        T::Arity: TryFromIter,
    {
        self.try_collect().unwrap()
    }

    pub fn try_collect(
        self,
    ) -> Result<
        Container<T::Arity, (ROQueryItem<'w, 's, Id>, ROQueryItem<'w, 's, Data>)>,
        HierarchicalError<T::Arity>,
    >
    where
        T::Arity: TryFromIter,
    {
        let query = self.query.into_readonly();
        let iter = query
            .iter_many_inner(self.collection.iter().flat_map(|it| it.iter()))
            .map(|it| (it.0, it.1));
        <T::Arity as TryFromIter>::try_from_iter(iter)
            .map_err(|e| HierarchicalError::WrongContainerSize(e))
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
        &self,
    ) -> impl Iterator<
        Item = (
            NodeId<T>,
            ROQueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>,
        ),
    > {
        self.parent_query
            .iter()
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

    /// Retrieves from world references to both parent and child by given *parent* id.
    pub fn get_down(
        &self,
        id: NodeId<T>,
    ) -> (
        ROQueryItem<'_, 's, ParentData>,
        ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>,
    )
    where
        T::Arity: TryFromIter,
    {
        self.try_get_down(id)
            .expect("Cannot collect children into container")
    }

    #[inline]
    pub fn get_children(
        &self,
        id: NodeId<T>,
    ) -> ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>
    where
        T::Arity: TryFromIter,
    {
        self.get_down(id).1
    }

    pub fn try_get_down(
        &self,
        id: NodeId<T>,
    ) -> Result<
        (
            ROQueryItem<'_, 's, ParentData>,
            ChildrenFetch<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>>,
        ),
        HierarchicalError<T::Arity>,
    >
    where
        T::Arity: TryFromIter,
    {
        let (_, parent, children) = self.parent_query.get(id.entity())?;
        Ok((
            parent,
            ChildrenFetch {
                collection: children,
                query: self.children_query.as_readonly(),
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
