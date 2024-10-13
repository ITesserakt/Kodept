pub use self::any_node::{AnyNode, AnyNodeD};
pub use self::children::tags;
pub use self::node_id::{AnyNodeId, AnyNodeKey, NodeId};
pub use self::children::HasChildrenMarker;
pub use self::syntax_tree::{
    dfs, subtree::SubSyntaxTree, SyntaxTree, SyntaxTreeBuilder,
};

pub(crate) use self::children::macros::with_children;
pub(crate) use self::identity::Identity;
pub(crate) use self::utils::*;

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
    fn set_id(&self, value: NodeId<Self>);
}

mod any_node;
mod children;
mod identity;
mod node_id;
mod syntax_tree;
mod utils;
pub mod node_props;
