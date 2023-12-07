use crate::node_id::NodeId;
use crate::traits::{IdProducer, Identifiable, Instantiable, IntoAst, Linker};
use crate::{impl_identifiable, BodiedFunctionDeclaration, ExpressionBlock, Operation, Type};
use derive_more::{From, IsVariant};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::BlockLevelNode;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Body {
    Block(ExpressionBlock),
    Simple(Box<BlockLevel>),
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BlockLevel {
    InitVar(InitializedVariable),
    Operation(Operation),
    Block(ExpressionBlock),
    Function(BodiedFunctionDeclaration),
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
    pub assigned_type: Option<Type>,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InitializedVariable {
    pub variable: Variable,
    pub expr: Operation,
    id: NodeId<Self>,
}

impl_identifiable! {
    Variable,
    InitializedVariable
}
node_group!(family: Body, nodes: [Body, ExpressionBlock, BlockLevel]);
node_group!(family: InitializedVariable, nodes: [InitializedVariable, Variable, Operation]);
node_group!(family: BlockLevel, nodes: [
    BlockLevel, InitializedVariable, Operation, ExpressionBlock, BodiedFunctionDeclaration
]);

impl Identifiable for BlockLevel {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            BlockLevel::InitVar(x) => x.get_id().cast(),
            BlockLevel::Operation(x) => x.get_id().cast(),
            BlockLevel::Block(x) => x.get_id().cast(),
            BlockLevel::Function(x) => x.get_id().cast(),
        }
    }
}

#[cfg(feature = "size-of")]
impl SizeOf for BlockLevel {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            BlockLevel::InitVar(x) => x.size_of_children(context),
            BlockLevel::Operation(x) => x.size_of_children(context),
            BlockLevel::Block(x) => x.size_of_children(context),
            BlockLevel::Function(x) => x.size_of_children(context),
        }
    }
}

impl IntoAst for rlt::Variable {
    type Output = Variable;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let (kind, id, ty) = match self {
            rlt::Variable::Immutable {
                id, assigned_type, ..
            } => (VariableKind::Immutable, id, assigned_type.as_ref()),
            rlt::Variable::Mutable {
                id, assigned_type, ..
            } => (VariableKind::Mutable, id, assigned_type.as_ref()),
        };
        let node = Variable {
            kind,
            name: context.get_chunk_located(id).to_string(),
            assigned_type: ty.map(|it| it.1.construct(context)),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::InitializedVariable {
    type Output = InitializedVariable;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = InitializedVariable {
            variable: self.variable.construct(context),
            expr: self.expression.construct(context),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl Identifiable for Body {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            Body::Block(x) => x.get_id().cast(),
            Body::Simple(x) => x.get_id().cast(),
        }
    }
}

impl IntoAst for rlt::Body {
    type Output = Body;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = match self {
            rlt::Body::Block(x) => x.construct(context).into(),
            rlt::Body::Simplified { expression, .. } => {
                Box::new(expression.construct(context)).into()
            }
        };
        context.link(node, self)
    }
}

impl IntoAst for BlockLevelNode {
    type Output = BlockLevel;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = match self {
            BlockLevelNode::InitVar(x) => x.construct(context).into(),
            BlockLevelNode::Block(x) => x.construct(context).into(),
            BlockLevelNode::Function(x) => x.construct(context).into(),
            BlockLevelNode::Operation(x) => x.construct(context).into(),
        };
        context.link(node, self)
    }
}

impl Instantiable for Variable {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            kind: self.kind.clone(),
            name: self.name.clone(),
            assigned_type: self
                .assigned_type
                .as_ref()
                .map(|it| it.new_instance(context)),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for InitializedVariable {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            variable: self.variable.new_instance(context),
            expr: self.expr.new_instance(context),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Body {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Body::Block(x) => x.new_instance(context).into(),
            Body::Simple(x) => Box::new(x.new_instance(context)).into(),
        }
    }
}

impl Instantiable for BlockLevel {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            BlockLevel::InitVar(x) => x.new_instance(context).into(),
            BlockLevel::Operation(x) => x.new_instance(context).into(),
            BlockLevel::Block(x) => x.new_instance(context).into(),
            BlockLevel::Function(x) => x.new_instance(context).into(),
        }
    }
}
