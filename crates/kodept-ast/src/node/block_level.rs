use std::fmt::Debug;

use derive_more::{IsVariant};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::BlockLevelNode;
use kodept_core::structure::span::CodeHolder;

use crate::graph::NodeId;
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::traits::{Linker, PopulateTree};
use crate::{node, BodyFnDecl, Exprs, Operation, Type, node_sub_enum};

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
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct VarDecl {
        pub kind: VariableKind,
        pub name: String,;
        pub assigned_type: Option<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
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

impl PopulateTree for rlt::Body {
    type Output = Body;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Body::Block(x) => x.convert(builder, context).cast(),
            rlt::Body::Simplified { expression, .. } => expression.convert(builder, context).cast(),
        }
    }
}

impl PopulateTree for BlockLevelNode {
    type Output = BlockLevel;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            BlockLevelNode::InitVar(x) => x.convert(builder, context).cast(),
            BlockLevelNode::Block(x) => x.convert(builder, context).cast(),
            BlockLevelNode::Function(x) => x.convert(builder, context).cast(),
            BlockLevelNode::Operation(x) => x.convert(builder, context).cast(),
        }
    }
}

impl PopulateTree for rlt::InitializedVariable {
    type Output = InitVar;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(InitVar::uninit())
            .with_children_from([&self.expression], context)
            .with_children_from([&self.variable], context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Variable {
    type Output = VarDecl;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let (kind, name, ty) = match self {
            rlt::Variable::Immutable {
                id, assigned_type, ..
            } => (VariableKind::Immutable, id, assigned_type),
            rlt::Variable::Mutable {
                id, assigned_type, ..
            } => (VariableKind::Mutable, id, assigned_type),
        };
        builder
            .add_node(VarDecl::uninit(
                kind,
                context.get_chunk_located(name).to_string(),
            ))
            .with_children_from(ty.as_ref().map(|x| &x.1), context)
            .with_rlt(context, self)
            .id()
    }
}
