use std::ops::{Deref, DerefMut};
use crate::prelude::{ASTNode, NodeId};
use crate::syntax_tree::prelude::AST;
use bevy_ecs::prelude::Component;
use crate::Str;

pub mod tags;

pub trait NodeProperty: Component {}

#[derive(Debug, Component)]
pub struct Node {
    pub kind: &'static str
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct Root;

#[derive(Debug, Component, Clone)]
pub struct Name {
    pub name: Str
}

pub trait HasProperty<Property: NodeProperty>: Sized {
    #[inline]
    fn ensure_has(id: NodeId, ast: &AST) -> bool {
        ast.contains::<Property>(id)
    }
}

pub trait RequireProperty<Property: NodeProperty> {}

impl Deref for Name {
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl DerefMut for Name {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.name
    }
}

impl<P: NodeProperty, T: RequireProperty<P>> HasProperty<P> for T {
    #[inline]
    fn ensure_has(id: NodeId, ast: &AST) -> bool {
        if cfg!(debug_assertions) && !ast.contains::<P>(id) {
            panic!(
                "Expected node {id} to has a required property `{}`",
                std::any::type_name::<P>()
            );
        } else {
            true
        }
    }
}

impl NodeProperty for Node {}
impl<T: ASTNode> RequireProperty<Node> for T {}
impl NodeProperty for Root {}

impl NodeProperty for Name {}
