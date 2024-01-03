use crate::graph::generic_node::GenericASTNode;
use crate::graph::traits::PopulateTree;
use crate::graph::{Identity, SyntaxTree};
use crate::node_id::NodeId;
use crate::traits::Linker;
use crate::{
    impl_identifiable_2, with_children, wrapper, BodiedFunctionDeclaration, ExpressionBlock,
    Operation, Type,
};
use derive_more::{From, Into, IsVariant};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::BlockLevelNode;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::{Context, SizeOf};
use std::fmt::Debug;

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper BlockLevel {
        func(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction,
        init_var(InitializedVariable) = GenericASTNode::InitializedVariable,
        operation(Operation) = GenericASTNode::Operation,
        block(Body) = GenericASTNode::Body
    }
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Body {
    Block(ExpressionBlock),
    Simple(Box<BlockLevel>),
}

#[derive(Debug, PartialEq, Clone, IsVariant)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum VariableKind {
    Immutable,
    Mutable,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Variable {
    pub kind: VariableKind,
    pub name: String,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InitializedVariable {
    id: NodeId<Self>,
}

impl_identifiable_2! {
    Variable,
    InitializedVariable
}

with_children!(InitializedVariable => {
    pub variable: Identity<Variable>
    pub expr: Identity<Operation>
});

with_children!(Variable => {
    pub assigned_type: Option<Type>
});

impl PopulateTree for rlt::Body {
    type Output = Body;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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

#[cfg(feature = "size-of")]
impl SizeOf for BlockLevel {
    fn size_of_children(&self, context: &mut Context) {
        self.0.size_of_children(context)
    }
}
