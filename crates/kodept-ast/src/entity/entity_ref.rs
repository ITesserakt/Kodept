use crate::prelude::{ASTNode, NodeId};
use crate::properties::{Node, NodeProperty, RequireProperty, SourceSpan};
use bevy_ecs::prelude::EntityRef;
use bevy_ecs::query::QueryData;
use std::convert::identity;
use std::ops::Deref;

#[derive(QueryData, Copy, Clone)]
#[query_data(derive(Copy, Clone))]
pub struct AnyNodeRef<'w> {
    inner: EntityRef<'w>,
}

#[derive(Copy, Clone)]
pub struct NodeRef<'w, T> {
    inner: EntityRef<'w>,
    node: T,
}

impl<'a> AnyNodeRefItem<'a, '_> {
    pub(crate) fn from_inner(item: EntityRef<'a>) -> Self {
        Self { inner: item }
    }
    
    #[deprecated]
    pub fn cast<T>(self) -> Option<NodeRef<'a, &'a T>>
    where
        T: ASTNode,
    {
        let value = self.inner.get::<T>()?;
        Some(NodeRef {
            inner: self.inner,
            node: value,
        })
    }

    #[inline(always)]
    pub fn get<T>(self) -> Option<NodeRef<'a, &'a T>>
    where
        T: ASTNode,
    {
        self.get_map(identity)
    }

    #[inline(always)]
    pub fn get_map<T, U>(self, f: impl FnOnce(&'a T) -> U) -> Option<NodeRef<'a, U>>
    where
        T: ASTNode,
    {
        let value = self.inner.get::<T>()?;
        Some(NodeRef {
            inner: self.inner,
            node: f(value),
        })
    }

    #[inline(always)]
    pub fn kind(&self) -> &'static str {
        self.inner.get::<Node>().unwrap().kind
    }
    
    pub fn span(&self) -> SourceSpan {
        *self.inner.get::<SourceSpan>().unwrap()
    }
    
    pub fn property<P: NodeProperty>(&self) -> Option<&P> {
        self.inner.get()
    } 
    
    pub fn id(&self) -> NodeId {
        self.inner.id().into()
    }
}

impl<T> NodeRef<'_, &T> {
    pub fn id(&self) -> NodeId<T> {
        self.inner.id().into()
    }
}

impl<'a, T> NodeRef<'a, &'a T> {
    pub fn property<P>(&self) -> &P
    where
        T: RequireProperty<P>,
        P: NodeProperty,
    {
        self.inner.get::<P>().expect("Node must have property")
    }
}

impl<'a, T> NodeRef<'a, T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> NodeRef<'a, U> {
        NodeRef {
            inner: self.inner,
            node: f(self.node),
        }
    }
}

impl<T> Deref for NodeRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
