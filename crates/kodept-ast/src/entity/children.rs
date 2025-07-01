use crate::arity::{Optional, Plural, Singular};
use crate::prelude::{ASTNode, NodeId};
use crate::relationship::NodeRelationship;
use crate::syntax_tree::children::HasChild;
use bevy_ecs::prelude::{Entity, Query, RelationshipTarget};
use bevy_ecs::relationship::Relationship;
use bevy_ecs::system::SystemParam;
use derive_more::{Display, Error};
use smallvec::SmallVec;
use std::convert::Infallible;
use std::error::Error;
use bevy_ecs::query::QueryFilter;

type Rel<T, U, Tag> = <T as NodeRelationship<U, Tag>>::Relationship;
type Target<T> = <T as Relationship>::RelationshipTarget;

#[derive(SystemParam)]
pub struct HierarchicalQuery<'w, 's, T, U, Tag = (), Filter = ()>
where
    T: HasChild<U, Tag>,
    T: ASTNode,
    U: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static
{
    parent_query: Query<'w, 's, (Entity, &'static T, &'static Target<Rel<T, U, Tag>>), Filter>,
    children_query: Query<'w, 's, (Entity, &'static U), Filter>,
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

impl<'w, 's, T, U, Tag, Filter> HierarchicalQuery<'w, 's, T, U, Tag, Filter>
where
    T: HasChild<U, Tag>,
    T: ASTNode,
    U: ASTNode,
    Tag: 'static,
    Filter: QueryFilter + 'static
{
    pub fn iter(&self) -> impl Iterator<Item = (NodeId<T>, NodeId<U>, &T, &U)> + '_ {
        self.parent_query.iter().flat_map(|parent| {
            self.children_query.iter_many(parent.2.iter()).map(move |child| {
                (
                    NodeId::from(parent.0),
                    NodeId::from(child.0),
                    parent.1,
                    child.1,
                )
            })
        })
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
