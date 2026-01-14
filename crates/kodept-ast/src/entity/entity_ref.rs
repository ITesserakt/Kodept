use crate::prelude::{ASTNode, IntoEnum, NodeId};
use crate::properties::{Node, NodeProperty, RequireProperty, SourceSpan};
use bevy_ecs::prelude::{ChildOf, EntityRef};
use bevy_ecs::query::QueryData;
use std::convert::identity;
use std::ops::Deref;
use bevy_ecs::relationship::Relationship;
use bevy_utils::prelude::DebugName;

#[derive(QueryData, Copy, Clone)]
#[query_data(derive(Copy, Clone))]
pub struct AnyNodeRef {
    inner: EntityRef<'static>,
}

#[derive(Copy, Clone)]
pub struct NodeRef<'w, T> {
    inner: EntityRef<'w>,
    node: T,
}

impl<'w> AnyNodeRefItem<'w, '_> {
    #[deprecated]
    pub fn cast<T>(self) -> Option<NodeRef<'w, &'w T>>
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
    pub fn get<T>(self) -> Option<NodeRef<'w, &'w T>>
    where
        T: ASTNode,
    {
        self.get_map(identity)
    }

    #[inline(always)]
    pub fn get_map<T, U>(self, f: impl FnOnce(&'w T) -> U) -> Option<NodeRef<'w, U>>
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
    pub fn kind(&self) -> &DebugName {
        &self.inner.get::<Node>().unwrap().kind
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
    
    pub fn parent(&self) -> Option<NodeId> {
        self.inner.get::<ChildOf>().map(|it| it.get().into())
    }
    
    pub fn to_enum<E>(self) -> Option<E>
    where 
        Self: IntoEnum<E>
    {
        IntoEnum::into_enum(self)
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
