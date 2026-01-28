use crate::arity::{Optional, Plural, Singular};
use crate::prelude::{ASTNode, NodeId};
use crate::relationship::NodeRelationship;
use crate::syntax_tree::children::{Family, HasChild};
use bevy_ecs::prelude::{Entity, Query, RelationshipTarget, With};
use bevy_ecs::query::{
    QueryData, QueryEntityError, QueryFilter, QueryManyIter, ROQueryItem, ReadOnlyQueryData,
};
use bevy_ecs::relationship::{Relationship, RelationshipSourceCollection};
use bevy_ecs::system::SystemParam;
use derive_more::{Display, Error, From};
use smallvec::SmallVec;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::iter::FusedIterator;
use std::marker::PhantomData;

type Rel<T, Tag> = <T as NodeRelationship<Tag, <T as Family<Tag>>::Arity>>::Relationship;
type Target<T> = <T as Relationship>::RelationshipTarget;
type Container<A, T> = <A as TryFromIter>::Container<T>;

pub struct Children<'w, 's, Data, T, Tag = (), Id = Entity, Filter = ()>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: From<Entity>,
{
    query: Query<'w, 's, (Entity, Data, &'static Rel<T, Tag>), Filter>,
    collection: &'w Target<Rel<T, Tag>>,
    _phantom: PhantomData<Id>,
}

pub struct ChildrenIter<'w, 's, Data, R, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    R: Relationship,
    Id: From<Entity>
{
    inner: QueryManyIter<'w, 's, (Entity, Data, &'static R), Filter,
        <<R::RelationshipTarget as RelationshipTarget>::Collection as RelationshipSourceCollection>::SourceIter<'w>,
    >,
    _phantom: PhantomData<Id>
}

#[derive(SystemParam)]
pub struct HierarchicalQuery<
    'w,
    's,
    T,
    Tag = (),
    ParentData = &'static T,
    ChildData = (),
    Filter = (),
> where
    T: Family<Tag>,
    T: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: QueryData + 'static,
    ChildData: QueryData + 'static,
{
    parent_query: Query<'w, 's, (Entity, ParentData, &'static Target<Rel<T, Tag>>), Filter>,
    children_query: Query<'w, 's, (Entity, ChildData, &'static Rel<T, Tag>)>,
}

#[derive(SystemParam)]
pub struct NarrowHierarchicalQuery<
    'w,
    's,
    T,
    U,
    Tag = (),
    ParentData = &'static T,
    ChildData = &'static U,
    Filter = (),
> where
    T: HasChild<U, Tag>,
    T: ASTNode,
    U: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
    ParentData: QueryData + 'static,
    ChildData: QueryData + 'static,
{
    parent_query: Query<'w, 's, (Entity, ParentData, &'static Target<Rel<T, Tag>>), Filter>,
    children_query: Query<'w, 's, (Entity, ChildData, &'static Rel<T, Tag>), With<U>>,
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
    ParentData: QueryData + 'static,
    ChildData: QueryData + 'static,
{
    #[allow(unsafe_code)]
    pub fn iter_by_layers(
        &self,
    ) -> impl Iterator<
        Item = (
            NodeId<T>,
            ROQueryItem<'_, 's, ParentData>,
            Children<'_, 's, ChildData::ReadOnly, T, Tag, Entity, ()>,
        ),
    > {
        self.parent_query
            .iter()
            .map(|(parent_id, parent_data, children)| {
                (
                    NodeId::from(parent_id),
                    parent_data,
                    Children {
                        collection: children,
                        query: self.children_query.as_readonly(),
                        _phantom: PhantomData,
                    },
                )
            })
    }
}

impl<'w, 's, Data, T, Tag, Id, Filter> IntoIterator for Children<'w, 's, Data, T, Tag, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: From<Entity>,
{
    type Item = (Id, ROQueryItem<'w, 's, Data>);
    type IntoIter = ChildrenIter<'w, 's, Data::ReadOnly, Rel<T, Tag>, Id, Filter>;

    fn into_iter(self) -> Self::IntoIter {
        let query = self.query.into_readonly();

        ChildrenIter {
            inner: query.iter_many_inner(self.collection.iter()),
            _phantom: PhantomData,
        }
    }
}

impl<'ww, 'w, 's, Data, T, Tag, Id, Filter> IntoIterator
    for &'ww Children<'w, 's, Data, T, Tag, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: From<Entity>,
{
    type Item = (Id, ROQueryItem<'ww, 's, Data>);
    type IntoIter = ChildrenIter<'ww, 's, Data::ReadOnly, Rel<T, Tag>, Id, Filter>;

    fn into_iter(self) -> Self::IntoIter {
        let query = self.query.as_readonly();
        ChildrenIter {
            inner: query.iter_many_inner(self.collection.iter()),
            _phantom: PhantomData,
        }
    }
}

impl<'w, 's, Data, Rel, Id, Filter> Iterator for ChildrenIter<'w, 's, Data, Rel, Id, Filter>
where
    Data: ReadOnlyQueryData,
    Filter: QueryFilter,
    Rel: Relationship,
    Id: From<Entity>,
{
    type Item = (Id, ROQueryItem<'w, 's, Data>);

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.inner.next()?;
        Some((Id::from(value.0), value.1))
    }
}

impl<'w, 's, Data, Rel, Id, Filter> FusedIterator for ChildrenIter<'w, 's, Data, Rel, Id, Filter>
where
    Data: ReadOnlyQueryData,
    Filter: QueryFilter,
    Rel: Relationship,
    Id: From<Entity>,
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
    Id: From<Entity>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let value = self.inner.next_back()?;
        Some((Id::from(value.0), value.1))
    }
}

impl<'w, 's, Data, T, Tag, Id, Filter> Children<'w, 's, Data, T, Tag, Id, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
    T: Family<Tag>,
    Id: From<Entity>,
{
    #[inline]
    pub fn iter(&self) -> ChildrenIter<'_, 's, Data::ReadOnly, Rel<T, Tag>, Id, Filter> {
        self.into_iter()
    }

    pub fn collect(self) -> Container<T::Arity, (Id, ROQueryItem<'w, 's, Data>)>
    where
        T::Arity: TryFromIter,
    {
        let query = self.query.into_readonly();
        let iter = query
            .iter_many_inner(self.collection.iter())
            .map(|it| (Id::from(it.0), it.1));
        <T::Arity as TryFromIter>::try_from_iter(iter).unwrap()
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
    ParentData: QueryData + 'static,
    ChildData: QueryData + 'static,
{
    pub fn iter_by_layers(
        &self,
    ) -> impl Iterator<
        Item = (
            NodeId<T>,
            ROQueryItem<'_, 's, ParentData>,
            Children<'_, 's, ChildData::ReadOnly, T, Tag, NodeId<U>, With<U>>,
        ),
    > {
        self.parent_query
            .iter()
            .map(|(parent_id, parent_data, children)| {
                (
                    NodeId::from(parent_id),
                    parent_data,
                    Children {
                        query: self.children_query.as_readonly(),
                        collection: children,
                        _phantom: PhantomData,
                    },
                )
            })
    }

    /// Retrieves from world references to both parent and child by given *parent* id.
    /// Essentially, this method costs one constant lookup into world and one iteration through all found children, so overall time complexity is `O(n)` where `n` is amount of children with respect to arity.
    /// If arity is [`Singular`] then `n == 1` and so on.
    ///
    /// # Panics
    ///
    /// Panics if amount of found children does not conform with the arity of this relationship.
    /// Or if there is no such parent by the given [`id`]
    pub fn get_down(
        &self,
        id: NodeId<T>,
    ) -> (
        ROQueryItem<'_, 's, ParentData>,
        Container<T::Arity, (NodeId<U>, ROQueryItem<'_, 's, ChildData>)>,
    )
    where
        T::Arity: TryFromIter,
    {
        self.try_get_down(id)
            .expect("Cannot collect children into container")
    }

    pub fn try_get_down(
        &self,
        id: NodeId<T>,
    ) -> Result<
        (
            ROQueryItem<'_, 's, ParentData>,
            Container<T::Arity, (NodeId<U>, ROQueryItem<'_, 's, ChildData>)>,
        ),
        HierarchicalError<T::Arity>,
    >
    where
        T::Arity: TryFromIter,
    {
        let (_, parent, children) = self.parent_query.get(id.entity())?;
        let iter = self
            .children_query
            .iter_many(children.iter())
            .map(|it| (it.0.into(), it.1));
        let container = <T::Arity as TryFromIter>::try_from_iter(iter)
            .map_err(HierarchicalError::WrongContainerSize)?;
        Ok((parent, container))
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
