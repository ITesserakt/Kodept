use crate::prelude::{ASTNode, NodeId};
use crate::syntax_tree::prelude::AST;
use bevy_ecs::prelude::Component;

pub mod tags;

pub trait NodeProperty: Component {}

#[derive(Debug, Component)]
pub struct Node {
    pub kind: &'static str
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct Root;

pub trait HasProperty<Property: NodeProperty>: Sized {
    #[inline]
    fn ensure_has(id: NodeId, ast: &AST) -> bool {
        ast.contains::<Property>(id)
    }
}

pub trait RequireProperty<Property: NodeProperty> {}

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
