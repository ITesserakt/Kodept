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
pub enum GenericASTNode {
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

pub type AnyNode = GenericASTNode;

#[allow(unsafe_code)]
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

#[allow(unsafe_code)]
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
    fn get_id(&self) -> GenericNodeId {
        functor_map!(GenericASTNode, self, |x| x.get_id().widen())
    }

    #[inline]
    fn set_id(&self, value: GenericNodeId) {
        functor_map!(GenericASTNode, self, |x| x.set_id(value.narrow()))
    }
}

impl GenericASTNode {
    #[inline]
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
        }
    }
}
