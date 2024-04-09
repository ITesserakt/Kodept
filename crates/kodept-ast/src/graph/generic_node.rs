use std::fmt::Debug;

use derive_more::{From, TryInto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::{ConvertibleToMut, ConvertibleToRef};

use crate::graph::changes::Change;
use crate::graph::{GhostToken, Identifiable, NodeId, SyntaxTree};
use crate::make_ast_node_adaptor;
use crate::*;

type Identity<T> = T;

make_ast_node_adaptor!(GenericASTNode, lifetimes: [], Identity, configs: [
    derive(Debug, PartialEq, From, TryInto),
    try_into(owned, ref, ref_mut),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize))
]);

#[macro_export]
macro_rules! generic_ast_node_map {
    ($self:expr, |$var:ident| $mapping:expr) => {
        $crate::functor_map!(GenericASTNode, $self, |$var| $mapping)
    };
}

pub trait NodeWithParent {
    type Parent;
}

#[allow(private_bounds)]
pub trait Node: Identifiable + Into<GenericASTNode> {
    fn parent<'b>(&self, tree: &'b SyntaxTree, token: &'b GhostToken) -> &'b Self::Parent
    where
        Self: NodeWithParent,
        GenericASTNode: ConvertibleToRef<Self::Parent>,
    {
        let id = self.get_id();
        tree.parent_of(id, token)
            .expect("NodeWithParent: contract violated")
            .try_as_ref()
            .expect("Node has wrong type")
    }

    fn parent_mut<'b>(
        &self,
        tree: &'b mut SyntaxTree,
        token: &'b mut GhostToken,
    ) -> &'b mut Self::Parent
    where
        Self: NodeWithParent,
        GenericASTNode: ConvertibleToMut<Self::Parent>,
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
    #[inline]
    fn get_id(&self) -> NodeId<Self> {
        generic_ast_node_map!(self, |x| x.get_id().cast())
    }

    #[inline]
    fn set_id(&mut self, value: NodeId<Self>) {
        generic_ast_node_map!(self, |x| x.set_id(value.cast()))
    }
}

impl GenericASTNode {
    pub fn name(&self) -> &'static str {
        match self {
            GenericASTNode::File(_) => "File",
            GenericASTNode::Module(_) => "Module",
            GenericASTNode::Struct(_) => "Struct",
            GenericASTNode::Enum(_) => "Enum",
            GenericASTNode::TypedParameter(_) => "TypedParameter",
            GenericASTNode::UntypedParameter(_) => "UntypedParameter",
            GenericASTNode::TypeName(_) => "TypeName",
            GenericASTNode::Variable(_) => "Variable",
            GenericASTNode::InitializedVariable(_) => "InitializedVariable",
            GenericASTNode::BodiedFunction(_) => "BodiedFunction",
            GenericASTNode::ExpressionBlock(_) => "ExpressionBlock",
            GenericASTNode::Application(_) => "Application",
            GenericASTNode::Lambda(_) => "Lambda",
            GenericASTNode::Reference(_) => "Reference",
            GenericASTNode::Access(_) => "Access",
            GenericASTNode::Number(_) => "Number",
            GenericASTNode::Char(_) => "Char",
            GenericASTNode::String(_) => "String",
            GenericASTNode::Tuple(_) => "Tuple",
            GenericASTNode::If(_) => "If",
            GenericASTNode::Elif(_) => "Elif",
            GenericASTNode::Else(_) => "Else",
            GenericASTNode::Binary(_) => "Binary",
            GenericASTNode::Unary(_) => "Unary",
            GenericASTNode::AbstractFunction(_) => "AbstractFunction",
            GenericASTNode::ProdType(_) => "ProdType",
            GenericASTNode::SumType(_) => "SumType",
        }
    }
}
