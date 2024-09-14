pub use self::any_node::{AnyNode, AnyNodeD, SubEnum};
pub use self::changes::{Change, ChangeSet};
pub use self::children::tags;
pub use self::node_id::{GenericNodeId, GenericNodeKey, NodeId};
pub use self::nodes::{PermTkn, RefNode};
pub use self::syntax_tree::{
    dfs, stage, subtree::SubSyntaxTree, SyntaxTree, SyntaxTreeBuilder, SyntaxTreeMutView,
};
pub use self::utils::TypedNodeCell;

pub(crate) use self::children::macros::with_children;
pub(crate) use self::children::HasChildrenMarker;
#[allow(unused_imports)]
pub(crate) use self::children::{ChildrenMut, ChildrenRef, ContainerT};
pub(crate) use self::identity::Identity;

pub(crate) trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
    fn set_id(&self, value: NodeId<Self>);
}

mod any_node;
mod changes;
mod children;
mod identity;
mod node_id;
mod nodes;
mod syntax_tree;
mod utils;
