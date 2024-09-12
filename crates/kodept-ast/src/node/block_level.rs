use std::fmt::Debug;

use derive_more::IsVariant;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::BlockLevelNode;
use kodept_core::structure::span::CodeHolder;

use crate::graph::Identity;
use crate::graph::SubSyntaxTree;
use crate::traits::PopulateTree;
use crate::{node, node_sub_enum, BodyFnDecl, Exprs, Operation, Type};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum BlockLevel {
        Fn(BodyFnDecl),
        InitVar(InitVar),
        Op(forward Operation),
        Block(Exprs)
    }
}

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum Body {
        Block(Exprs),
        Simple(forward BlockLevel)
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct VarDecl {
        pub kind: VariableKind,
        pub name: String,;
        pub assigned_type: Option<Type>,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct InitVar {
        ;
        pub variable: Identity<VarDecl>,
        pub expr: Identity<Operation>,
    }
}

#[derive(Debug, PartialEq, Clone, IsVariant)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum VariableKind {
    Immutable,
    Mutable,
}

impl<'a> PopulateTree<'a> for &'a rlt::Body {
    type Root = Body;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            rlt::Body::Block(x) => x.convert(context).cast(),
            rlt::Body::Simplified { expression, .. } => expression.convert(context).cast(),
        }
    }
}

impl<'a> PopulateTree<'a> for &'a BlockLevelNode {
    type Root = BlockLevel;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            BlockLevelNode::InitVar(x) => x.convert(context).cast(),
            BlockLevelNode::Block(x) => x.convert(context).cast(),
            BlockLevelNode::Function(x) => x.convert(context).cast(),
            BlockLevelNode::Operation(x) => x.convert(context).cast(),
        }
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::InitializedVariable {
    type Root = InitVar;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        SubSyntaxTree::new(InitVar::uninit().with_rlt(self))
            .with_children_from([&self.expression], context)
            .with_children_from([&self.variable], context)
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Variable {
    type Root = VarDecl;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        let (kind, name, ty) = match self {
            rlt::Variable::Immutable {
                id, assigned_type, ..
            } => (VariableKind::Immutable, id, assigned_type),
            rlt::Variable::Mutable {
                id, assigned_type, ..
            } => (VariableKind::Mutable, id, assigned_type),
        };
        SubSyntaxTree::new(
            VarDecl::uninit(kind, context.get_chunk_located(name).to_string()).with_rlt(self),
        )
        .with_children_from(ty.as_ref().map(|x| &x.1), context)
    }
}
