use crate::arity::{Optional, Plural, Singular};
use crate::prelude::{ASTNode, NodeId};
use crate::relationship::NodeRelationship;
use crate::syntax_tree::children::HasChild;
use bevy_ecs::prelude::{Entity, Query, RelationshipTarget};
use bevy_ecs::query::{QueryEntityError, QueryFilter};
use bevy_ecs::relationship::Relationship;
use bevy_ecs::system::SystemParam;
use derive_more::{Display, Error, From};
use smallvec::SmallVec;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

type Rel<T, U, Tag> = <T as NodeRelationship<U, Tag>>::Relationship;
type Target<T> = <T as Relationship>::RelationshipTarget;
type Container<A, T> = <A as TryFromIter>::Container<T>;

#[derive(SystemParam)]
pub struct HierarchicalQuery<'w, 's, T, U, Tag = (), Filter = ()>
where
    T: HasChild<U, Tag>,
    T: ASTNode,
    U: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
{
    parent_query: Query<'w, 's, (Entity, &'static T, &'static Target<Rel<T, U, Tag>>), Filter>,
    children_query: Query<'w, 's, (Entity, &'static U, &'static Rel<T, U, Tag>)>,
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

impl<'w, 's, T, U, Tag, Filter> HierarchicalQuery<'w, 's, T, U, Tag, Filter>
where
    T: HasChild<U, Tag>,
    T: ASTNode,
    U: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static,
{
    pub fn iter(&self) -> impl Iterator<Item = (NodeId<T>, NodeId<U>, &T, &U)> + '_ {
        self.parent_query.iter().flat_map(|parent| {
            self.children_query
                .iter_many(parent.2.iter())
                .map(move |child| {
                    (
                        NodeId::from(parent.0),
                        NodeId::from(child.0),
                        parent.1,
                        child.1,
                    )
                })
        })
    }

    pub fn iter_by_layers(
        &self,
    ) -> impl Iterator<Item = (NodeId<T>, &T, impl Iterator<Item = (NodeId<U>, &U)>)> {
        self.parent_query.iter().map(|parent| {
            (
                NodeId::from(parent.0),
                parent.1,
                self.children_query
                    .iter_many(parent.2.iter())
                    .map(move |child| (NodeId::from(child.0), child.1)),
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
    pub fn get_down(&self, id: NodeId<T>) -> (&T, Container<T::Arity, (NodeId<U>, &U)>)
    where
        T::Arity: TryFromIter,
        <T::Arity as TryFromIter>::Error: Debug,
    {
        self.try_get_down(id)
            .expect("Cannot collect children into container")
    }

    pub fn try_get_down(
        &self,
        id: NodeId<T>,
    ) -> Result<(&T, Container<T::Arity, (NodeId<U>, &U)>), HierarchicalError<T::Arity>>
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
    pub fn get_up(&self, id: NodeId<U>) -> Result<(NodeId<T>, &T, &U), QueryEntityError> {
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
    type Container<T> = SmallVec<[T; 8]>;

    type Error = Infallible;

    #[inline]
    fn try_from_iter<T>(
        iter: impl IntoIterator<Item = T>,
    ) -> Result<Self::Container<T>, Self::Error> {
        Ok(SmallVec::from_iter(iter))
    }
}
