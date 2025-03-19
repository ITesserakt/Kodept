use crate::node_id::Erase;
use crate::prelude::{ASTNode, AnyNodeRefItem, FromEnum, IntoEnum, NodeId, NodeRef, TryFromIter};
use crate::properties::{Node, Root};
use crate::syntax_tree::children::HasChild;
use bevy_ecs::prelude::{ChildOf, Children, Entity, EntityRef, Query, Single, With};
use bevy_ecs::query::{QueryEntityError, QueryFilter, ReadOnlyQueryData};
use bevy_ecs::system::SystemParam;
use derive_more::{Display, Error, From};
use std::collections::VecDeque;
use std::error::Error;
use bevy_ecs::relationship::Relationship;

type InnerQueryData<T> = (T, Option<&'static Children>, Option<&'static ChildOf>);
type InnerQuery<'w, 's, T, Filter> = Query<'w, 's, InnerQueryData<T>, (With<Node>, Filter)>;

#[derive(SystemParam)]
pub struct ASTQuery<'w, 's, Filter = (), T = EntityRef<'static>>
where
    T: 'static + ReadOnlyQueryData,
    Filter: 'static + QueryFilter,
{
    inner: InnerQuery<'w, 's, T, Filter>,
    root: Single<'w, Entity, With<Root>>,
}

#[derive(Debug, Display, Error, From)]
pub enum QueryError<E: Error> {
    Inner(#[error(source)] E),
    #[from(ignore)]
    NotFound(#[error(not(source))] Entity),
}

impl<T, F> ASTQuery<'_, '_, F, T>
where
    T: 'static + ReadOnlyQueryData,
    F: 'static + QueryFilter,
{
    #[inline]
    pub fn get(&self, id: impl Erase) -> Result<T::Item<'_>, QueryError<QueryEntityError>> {
        let (value, _, _) = self.inner.get(id.erase().into())?;
        Ok(value)
    }

    pub fn contains(&self, id: impl Erase) -> bool {
        self.inner.contains(id.erase().into())
    }

    pub fn node_count(&self) -> usize {
        self.inner.iter().count()
    }

    pub fn iter(&self) -> impl Iterator<Item = T::Item<'_>> {
        self.inner.iter().map(|(value, _, _)| value)
    }
    
    pub fn root(&self) -> NodeId {
        NodeId::from(*self.root)
    }
}

impl<F> ASTQuery<'_, '_, F, EntityRef<'static>>
where
    F: 'static + QueryFilter,
{
    pub fn get_as<T>(&self, id: impl Into<NodeId<T>>) -> Result<&T, QueryError<QueryEntityError>>
    where
        T: ASTNode,
    {
        let id = id.into().into();
        let (value, _, _) = self.inner.get(id)?;
        value.get::<T>().ok_or(QueryError::NotFound(id))
    }

    pub fn iter_as<T: ASTNode>(&self) -> impl Iterator<Item = NodeRef<&T>> {
        self.inner
            .iter()
            .filter_map(|(node, _, _)| AnyNodeRefItem::from_inner(node).get())
    }

    pub fn children_as<T, U, Tag>(
        &self,
        id: impl Into<NodeId<T>>,
    ) -> Result<
        <T::Arity as TryFromIter>::Container<NodeRef<&U>>,
        QueryError<<T::Arity as TryFromIter>::Error>,
    >
    where
        T: ASTNode + HasChild<U, Tag, Arity: TryFromIter>,
        U: ASTNode,
        Tag: Send + Sync + 'static,
    {
        let id = id.into().into();
        let (_, children, _) = self.inner.get(id).map_err(|_| QueryError::NotFound(id))?;
        let iter = children
            .into_iter()
            .flatten()
            .filter_map(|it| self.inner.get(*it).ok())
            .filter_map(|it| AnyNodeRefItem::from_inner(it.0).get());
        Ok(T::Arity::try_from_iter(iter)?)
    }

    pub fn parent_as(
        &self,
        id: impl Erase,
    ) -> Result<AnyNodeRefItem, QueryError<QueryEntityError>> {
        let id = id.erase().into();
        let (_, _, parent) = self.inner.get(id)?;
        if let Some(parent) = parent {
            let (node, _, _) = self.inner.get(parent.get())?;
            return Ok(AnyNodeRefItem::from_inner(node));
        }
        Err(QueryError::NotFound(id))
    }

    pub fn iter_enum<'a, E: FromEnum<'a>>(&'a self) -> impl Iterator<Item = E> + 'a {
        self.inner
            .iter()
            .filter_map(|it| AnyNodeRefItem::from_inner(it.0).into_enum())
    }
}

impl<'w, F> ASTQuery<'w, '_, F, EntityRef<'_>>
where
    F: 'static + QueryFilter,
{
    /// Iterates through all nodes from node with `root` property via bfs
    pub fn iter_descendants(&self) -> impl Iterator<Item = AnyNodeRefItem<'_, 'w>> {
        let mut queue = VecDeque::from([*self.root]);
        std::iter::from_fn(move || {
            let element = queue.pop_front()?;
            let (node, children, _) = self.inner.get(element).unwrap();
            queue.extend(children.into_iter().flatten().copied());

            Some(AnyNodeRefItem::from_inner(node))
        })
    }

    pub fn iter_ancestors(
        &self,
        start: impl Erase<Entity>,
    ) -> impl Iterator<Item = AnyNodeRefItem<'_, 'w>> {
        let mut current = Some(start.erase());
        std::iter::from_fn(move || {
            let (node, _, parent) = self.inner.get(current?).unwrap();
            current = parent.map(|it| it.get());
            Some(AnyNodeRefItem::from_inner(node))
        })
    }
}
