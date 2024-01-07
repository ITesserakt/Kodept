use derive_more::{From, TryInto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use std::fmt::Debug;

use crate::graph::changes::Change;
use crate::graph::{GhostToken, Identifiable, NodeId, SyntaxTree};
use crate::make_ast_node_adaptor;
use crate::*;

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

pub trait Node: Identifiable + Into<GenericASTNode> {
    fn parent<'b>(&self, tree: &'b SyntaxTree, token: &'b GhostToken) -> &'b Self::Parent
    where
        Self: NodeWithParent,
        for<'a> &'a Self::Parent: TryFrom<&'a GenericASTNode>,
        for<'a> <&'a Self::Parent as TryFrom<&'a GenericASTNode>>::Error: Debug,
    {
        let id = self.get_id();
        tree.parent_of(id, token)
    }

    fn parent_mut<'b>(
        &self,
        tree: &'b mut SyntaxTree,
        token: &'b mut GhostToken,
    ) -> &'b mut Self::Parent
    where
        Self: NodeWithParent,
        for<'a> &'a mut Self::Parent: TryFrom<&'a mut GenericASTNode>,
        for<'a> <&'a mut Self::Parent as TryFrom<&'a mut GenericASTNode>>::Error: Debug,
    {
        let id = self.get_id();
        tree.parent_of_mut(id, token)
    }

    fn replace_with(&self, other: Self) -> Change {
        Change::Replace {
            from_id: self.get_id().cast(),
            to: other.into(),
        }
    }

    fn remove(&self) -> Change {
        Change::DeleteSelf {
            node_id: self.get_id().cast(),
        }
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

impl Identifiable for GenericASTNode {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            GenericASTNode::File(x) => x.get_id().cast(),
            GenericASTNode::Module(x) => x.get_id().cast(),
            GenericASTNode::Struct(x) => x.get_id().cast(),
            GenericASTNode::Enum(x) => x.get_id().cast(),
            GenericASTNode::TypedParameter(x) => x.get_id().cast(),
            GenericASTNode::UntypedParameter(x) => x.get_id().cast(),
            GenericASTNode::TypeName(x) => x.get_id().cast(),
            GenericASTNode::Variable(x) => x.get_id().cast(),
            GenericASTNode::InitializedVariable(x) => x.get_id().cast(),
            GenericASTNode::BodiedFunction(x) => x.get_id().cast(),
            GenericASTNode::ExpressionBlock(x) => x.get_id().cast(),
            GenericASTNode::Application(x) => x.get_id().cast(),
            GenericASTNode::Lambda(x) => x.get_id().cast(),
            GenericASTNode::Reference(x) => x.get_id().cast(),
            GenericASTNode::Access(x) => x.get_id().cast(),
            GenericASTNode::Number(x) => x.get_id().cast(),
            GenericASTNode::Char(x) => x.get_id().cast(),
            GenericASTNode::String(x) => x.get_id().cast(),
            GenericASTNode::Tuple(x) => x.get_id().cast(),
            GenericASTNode::If(x) => x.get_id().cast(),
            GenericASTNode::Elif(x) => x.get_id().cast(),
            GenericASTNode::Else(x) => x.get_id().cast(),
            GenericASTNode::Binary(x) => x.get_id().cast(),
            GenericASTNode::Unary(x) => x.get_id().cast(),
            GenericASTNode::AbstractFunction(x) => x.get_id().cast(),
            GenericASTNode::ProdType(x) => x.get_id().cast(),
            GenericASTNode::SumType(x) => x.get_id().cast(),
        }
    }

    fn set_id(&mut self, value: NodeId<Self>) {
        match self {
            GenericASTNode::File(x) => x.set_id(value.cast()),
            GenericASTNode::Module(x) => x.set_id(value.cast()),
            GenericASTNode::Struct(x) => x.set_id(value.cast()),
            GenericASTNode::Enum(x) => x.set_id(value.cast()),
            GenericASTNode::TypedParameter(x) => x.set_id(value.cast()),
            GenericASTNode::UntypedParameter(x) => x.set_id(value.cast()),
            GenericASTNode::TypeName(x) => x.set_id(value.cast()),
            GenericASTNode::Variable(x) => x.set_id(value.cast()),
            GenericASTNode::InitializedVariable(x) => x.set_id(value.cast()),
            GenericASTNode::BodiedFunction(x) => x.set_id(value.cast()),
            GenericASTNode::ExpressionBlock(x) => x.set_id(value.cast()),
            GenericASTNode::Application(x) => x.set_id(value.cast()),
            GenericASTNode::Lambda(x) => x.set_id(value.cast()),
            GenericASTNode::Reference(x) => x.set_id(value.cast()),
            GenericASTNode::Access(x) => x.set_id(value.cast()),
            GenericASTNode::Number(x) => x.set_id(value.cast()),
            GenericASTNode::Char(x) => x.set_id(value.cast()),
            GenericASTNode::String(x) => x.set_id(value.cast()),
            GenericASTNode::Tuple(x) => x.set_id(value.cast()),
            GenericASTNode::If(x) => x.set_id(value.cast()),
            GenericASTNode::Elif(x) => x.set_id(value.cast()),
            GenericASTNode::Else(x) => x.set_id(value.cast()),
            GenericASTNode::Binary(x) => x.set_id(value.cast()),
            GenericASTNode::Unary(x) => x.set_id(value.cast()),
            GenericASTNode::AbstractFunction(x) => x.set_id(value.cast()),
            GenericASTNode::ProdType(x) => x.set_id(value.cast()),
            GenericASTNode::SumType(x) => x.set_id(value.cast()),
        }
    }
}
