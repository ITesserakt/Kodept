use crate::prelude::ASTNode;
use crate::Str;
use bevy_ecs::prelude::Component;
use derive_more::{Display, From, Into};
use std::ops::{Deref, DerefMut};

pub trait NodeProperty: Component {}

#[derive(Debug, Component, Display)]
pub struct Node {
    pub kind: &'static str
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct Root;

#[derive(Debug, Component, Clone, From, Into, Display)]
#[component(storage = "SparseSet")]
pub struct Name(pub Str);

pub trait HasProperty<Property: NodeProperty>: Sized {
}

pub trait RequireProperty<Property: NodeProperty> {}

impl Deref for Name {
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Name {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P: NodeProperty, T: RequireProperty<P>> HasProperty<P> for T {
}

impl NodeProperty for Node {}
impl<T: ASTNode> RequireProperty<Node> for T {}
impl NodeProperty for Root {}

impl NodeProperty for Name {}
