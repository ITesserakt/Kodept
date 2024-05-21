use std::fmt::Debug;

use derive_more::{From, TryInto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::*;
use crate::graph::Identifiable;
use crate::graph::node_id::GenericNodeId;

#[derive(Debug, PartialEq, From, TryInto)]
#[try_into(owned, ref, ref_mut)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AnyNode {
    File(FileDeclaration),
    Module(ModuleDeclaration),
    Struct(StructDeclaration),
    Enum(EnumDeclaration),
    TypedParameter(TypedParameter),
    UntypedParameter(UntypedParameter),
    TypeName(TypeName),
    Variable(Variable),
    InitializedVariable(InitializedVariable),
    BodiedFunction(BodiedFunctionDeclaration),
    ExpressionBlock(ExpressionBlock),
    Application(Application),
    Lambda(Lambda),
    Reference(Reference),
    Access(Access),
    Number(NumberLiteral),
    Char(CharLiteral),
    String(StringLiteral),
    Tuple(TupleLiteral),
    If(IfExpression),
    Elif(ElifExpression),
    Else(ElseExpression),
    Binary(Binary),
    Unary(Unary),
    AbstractFunction(AbstractFunctionDeclaration),
    ProdType(ProdType),
}

#[deprecated]
pub type GenericASTNode = AnyNode;

#[allow(unsafe_code)]
/// # Safety
/// Implement only for `#repr(transparent)` structs
pub unsafe trait NodeUnion: Sized {
    fn contains(node: &AnyNode) -> bool;

    #[inline]
    fn wrap(node: &AnyNode) -> &Self {
        debug_assert!(Self::contains(node));
        unsafe { std::mem::transmute(node) }
    }

    #[inline]
    fn wrap_mut(node: &mut AnyNode) -> &mut Self {
        debug_assert!(Self::contains(node));
        unsafe { std::mem::transmute(node) }
    }
}

#[allow(unsafe_code)]
unsafe impl NodeUnion for AnyNode {
    #[inline]
    fn contains(_node: &AnyNode) -> bool {
        true
    }

    #[inline]
    fn wrap(node: &AnyNode) -> &Self {
        node
    }

    #[inline]
    fn wrap_mut(node: &mut AnyNode) -> &mut Self {
        node
    }
}

impl Identifiable for AnyNode {
    #[inline]
    fn get_id(&self) -> GenericNodeId {
        functor_map!(AnyNode, self, |x| x.get_id().widen())
    }

    #[inline]
    fn set_id(&self, value: GenericNodeId) {
        functor_map!(AnyNode, self, |x| x.set_id(value.narrow()))
    }
}

impl AnyNode {
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            AnyNode::File(_) => "File",
            AnyNode::Module(_) => "Module",
            AnyNode::Struct(_) => "Struct",
            AnyNode::Enum(_) => "Enum",
            AnyNode::TypedParameter(_) => "TypedParameter",
            AnyNode::UntypedParameter(_) => "UntypedParameter",
            AnyNode::TypeName(_) => "TypeName",
            AnyNode::Variable(_) => "Variable",
            AnyNode::InitializedVariable(_) => "InitializedVariable",
            AnyNode::BodiedFunction(_) => "BodiedFunction",
            AnyNode::ExpressionBlock(_) => "ExpressionBlock",
            AnyNode::Application(_) => "Application",
            AnyNode::Lambda(_) => "Lambda",
            AnyNode::Reference(_) => "Reference",
            AnyNode::Access(_) => "Access",
            AnyNode::Number(_) => "Number",
            AnyNode::Char(_) => "Char",
            AnyNode::String(_) => "String",
            AnyNode::Tuple(_) => "Tuple",
            AnyNode::If(_) => "If",
            AnyNode::Elif(_) => "Elif",
            AnyNode::Else(_) => "Else",
            AnyNode::Binary(_) => "Binary",
            AnyNode::Unary(_) => "Unary",
            AnyNode::AbstractFunction(_) => "AbstractFunction",
            AnyNode::ProdType(_) => "ProdType",
        }
    }
}
