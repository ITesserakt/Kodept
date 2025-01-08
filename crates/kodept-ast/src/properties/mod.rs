use crate::prelude::{ASTNode, NodeId};
use crate::syntax_tree::prelude::AST;
use bevy_ecs::prelude::Component;

pub mod tags;

pub trait NodeProperty: Component {}

#[derive(Debug, Component)]
pub struct Node;

pub trait HasProperty<Property: NodeProperty>: Sized {
    #[inline]
    fn ensure_has(id: NodeId<Self>, ast: &AST) -> bool {
        ast.contains::<Property>(id.cast())
    }
}

pub trait RequireProperty<Property: NodeProperty> {}

impl<P: NodeProperty, T: RequireProperty<P>> HasProperty<P> for T {
    #[inline]
    fn ensure_has(id: NodeId<Self>, ast: &AST) -> bool {
        if cfg!(debug_assertions) && !ast.contains::<P>(id.cast()) {
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
