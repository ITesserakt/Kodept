use std::ops::Deref;
use bevy_ecs::prelude::EntityRef;
use bevy_ecs::query::QueryData;
use crate::prelude::{ASTNode, NodeId};
use crate::properties::{NodeProperty, RequireProperty};

#[derive(QueryData, Copy, Clone)]
#[query_data(derive(Copy, Clone))]
pub struct AnyNodeRef<'w> {
    inner: EntityRef<'w>
}

#[derive(Copy, Clone)]
pub struct NodeRef<'w, T> {
    inner: EntityRef<'w>,
    node: &'w T
}

impl<'a, 'w> AnyNodeRefItem<'a, 'w> {
    pub fn cast<T>(self) -> Option<NodeRef<'a, T>>
    where
        T: ASTNode
    {
        let value = self.inner.get::<T>()?;
        Some(NodeRef { inner: self.inner, node: value })
    }

    pub fn id(&self) -> NodeId {
        self.inner.id()
    }
}

impl<T> NodeRef<'_, T> {
    pub fn property<P>(&self) -> &P
    where
        T: RequireProperty<P>,
        P: NodeProperty
    {
        self.inner.get::<P>().expect("Node must have property")
    }

    pub fn id(&self) -> NodeId {
        self.inner.id()
    }
}

impl<T> Deref for NodeRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.node
    }
}
