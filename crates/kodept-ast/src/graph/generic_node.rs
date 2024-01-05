use derive_more::{From, TryInto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use crate::*;
use crate::graph::{GhostToken, Identifiable, SyntaxTree};
use crate::make_ast_node_adaptor;

type Identity<T> = T;

make_ast_node_adaptor!(GenericASTNode, lifetimes: [], Identity, configs: [
    derive(Debug, PartialEq, From, TryInto),
    try_into(owned, ref, ref_mut),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize)),
    cfg_attr(feature = "size-of", derive(SizeOf))
]);

pub trait NodeWithParent {
    type Parent;
}

pub trait Node: Identifiable {
    fn parent<'b>(&self, tree: &'b SyntaxTree, token: &'b GhostToken) -> &'b Self::Parent
    where
        Self: NodeWithParent + 'static,
        for<'a> &'a Self::Parent: TryFrom<&'a GenericASTNode>,
    {
        let id = self.get_id();
        tree.parent_of(id, token)
    }

    fn parent_mut<'b>(&self, tree: &'b mut SyntaxTree) -> &'b mut Self::Parent
    where
        Self: NodeWithParent + 'static,
        for<'a> &'a mut Self::Parent: TryFrom<&'a mut GenericASTNode>,
    {
        let id = self.get_id();
        tree.parent_of_mut(id)
    }
}

/// # Safety
/// Implement only for `#repr(transparent)` structs
pub unsafe trait NodeUnion: Sized {
    fn contains(node: &GenericASTNode) -> bool;

    #[inline]
    fn wrap(node: &GenericASTNode) -> &Self {
        debug_assert!(Self::contains(node));
        unsafe { std::mem::transmute(node) }
    }

    #[inline]
    fn wrap_mut(node: &mut GenericASTNode) -> &mut Self {
        debug_assert!(Self::contains(node));
        unsafe { std::mem::transmute(node) }
    }
}

unsafe impl NodeUnion for GenericASTNode {
    #[inline]
    fn contains(_node: &GenericASTNode) -> bool {
        true
    }

    #[inline]
    fn wrap(node: &GenericASTNode) -> &Self {
        node
    }

    #[inline]
    fn wrap_mut(node: &mut GenericASTNode) -> &mut Self {
        node
    }
}
