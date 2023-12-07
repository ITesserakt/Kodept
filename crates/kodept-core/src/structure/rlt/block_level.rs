use crate::code_point::CodePoint;
use crate::structure::rlt::BodiedFunction;
use crate::structure::{
    rlt::expression::{ExpressionBlock, Operation},
    rlt::new_types::*,
    rlt::Type,
    Located,
};
use derive_more::From;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Body {
    Block(ExpressionBlock),
    Simplified {
        flow: Symbol,
        expression: BlockLevelNode,
    },
}

#[derive(Clone, Debug, PartialEq, From)]
pub enum BlockLevelNode {
    InitVar(InitializedVariable),
    Block(ExpressionBlock),
    Function(BodiedFunction),
    Operation(Operation),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Variable {
    Immutable {
        keyword: Symbol,
        id: Identifier,
        assigned_type: Option<(Symbol, Type)>,
    },
    Mutable {
        keyword: Symbol,
        id: Identifier,
        assigned_type: Option<(Symbol, Type)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct InitializedVariable {
    pub variable: Variable,
    pub equals: Symbol,
    pub expression: Operation,
}

#[cfg(feature = "size-of")]
impl ::size_of::SizeOf for BlockLevelNode {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            Self::InitVar(x) => x.size_of_children(context),
            Self::Block(x) => x.size_of_children(context),
            Self::Function(x) => x.size_of_children(context),
            Self::Operation(x) => x.size_of_children(context),
        }
    }
}

impl Located for Variable {
    fn location(&self) -> CodePoint {
        match self {
            Variable::Immutable { id, .. } => id.location(),
            Variable::Mutable { id, .. } => id.location(),
        }
    }
}

impl Located for InitializedVariable {
    fn location(&self) -> CodePoint {
        self.variable.location()
    }
}

impl Located for Body {
    fn location(&self) -> CodePoint {
        match self {
            Body::Block(x) => x.location(),
            Body::Simplified { expression, .. } => expression.location(),
        }
    }
}

impl Located for BlockLevelNode {
    fn location(&self) -> CodePoint {
        match self {
            BlockLevelNode::InitVar(x) => x.location(),
            BlockLevelNode::Block(x) => x.location(),
            BlockLevelNode::Function(x) => x.location(),
            BlockLevelNode::Operation(x) => x.location(),
        }
    }
}
