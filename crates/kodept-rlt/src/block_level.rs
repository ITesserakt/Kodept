use crate::new_types::{Identifier, Keyword, Symbol};
use crate::prelude::{BodiedFunction, ExpressionBlock, Operation, Type};
use derive_more::From;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;

#[derive(Debug, Clone, PartialEq)]
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
pub enum Variable {
    Immutable {
        keyword: Keyword,
        id: Identifier,
        assigned_type: Option<(Symbol, Type)>,
    },
    Mutable {
        keyword: Keyword,
        id: Identifier,
        assigned_type: Option<(Symbol, Type)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct InitializedVariable {
    pub variable: Variable,
    pub equals: Symbol,
    pub expression: Operation,
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
