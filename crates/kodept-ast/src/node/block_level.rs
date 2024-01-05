use std::fmt::Debug;

use derive_more::{From, Into, IsVariant};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::rlt;
use kodept_core::structure::rlt::BlockLevelNode;
use kodept_core::structure::span::CodeHolder;

use crate::{BodiedFunctionDeclaration, ExpressionBlock, node, Operation, Type, wrapper};
use crate::graph::{GenericASTNode, NodeUnion};
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::graph::NodeId;
use crate::traits::{Linker, PopulateTree};

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper BlockLevel {
        func(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => Some(x),
        init_var(InitializedVariable) = GenericASTNode::InitializedVariable(x) => Some(x),
        operation(Operation) = n if Operation::contains(n) => n.try_into().ok(),
    }
}

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Body {
        block(ExpressionBlock) = GenericASTNode::ExpressionBlock(x) => Some(x),
        simple(BlockLevel) = x if BlockLevel::contains(x) => x.try_into().ok(),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Variable {
        pub kind: VariableKind,
        pub name: String,;
        pub assigned_type: Option<Type>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct InitializedVariable {
        ;
        pub variable: Identity<Variable>,
        pub expr: Identity<Operation>,
    }
}

#[derive(Debug, PartialEq, Clone, IsVariant)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum VariableKind {
    Immutable,
    Mutable,
}

impl BlockLevel {
    pub fn as_body(&self) -> Option<&Body> {
        match self {
            BlockLevel(n @ GenericASTNode::ExpressionBlock(_)) => Some(Body::wrap(n)),
            BlockLevel(n @ GenericASTNode::InitializedVariable(_)) => Some(Body::wrap(n)),
            BlockLevel(n @ GenericASTNode::BodiedFunction(_)) => Some(Body::wrap(n)),
            BlockLevel(n) if Operation::contains(n) => Some(Body::wrap(n)),
            _ => None,
        }
    }

    pub fn as_body_mut(&mut self) -> Option<&mut Body> {
        match self {
            BlockLevel(n @ GenericASTNode::ExpressionBlock(_)) => Some(Body::wrap_mut(n)),
            BlockLevel(n @ GenericASTNode::InitializedVariable(_)) => Some(Body::wrap_mut(n)),
            BlockLevel(n @ GenericASTNode::BodiedFunction(_)) => Some(Body::wrap_mut(n)),
            BlockLevel(n) if Operation::contains(n) => Some(Body::wrap_mut(n)),
            _ => None,
        }
    }
}

impl PopulateTree for rlt::Body {
    type Output = Body;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Body::Block(x) => x.convert(builder, context).cast(),
            rlt::Body::Simplified { expression, .. } => expression.convert(builder, context).cast(),
        }
    }
}

impl PopulateTree for BlockLevelNode {
    type Output = BlockLevel;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
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
    type Output = InitializedVariable;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(InitializedVariable {
                id: Default::default(),
            })
            .with_children_from([&self.expression], context)
            .with_children_from([&self.variable], context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Variable {
    type Output = Variable;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
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
            .add_node(Variable {
                kind,
                name: context.get_chunk_located(name).to_string(),
                id: Default::default(),
            })
            .with_children_from(ty.as_ref().map(|x| &x.1), context)
            .with_rlt(context, self)
            .id()
    }
}
