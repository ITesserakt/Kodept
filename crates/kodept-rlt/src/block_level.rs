use crate::new_types::{Identifier, Keyword, Symbol};
use crate::prelude::{BodiedFunction, ExpressionBlock, Operation, Type};
use derive_more::From;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Body {
    Block(ExpressionBlock),
    Simplified {
        flow: Symbol,
        expression: BlockLevelNode,
    },
}

#[derive(Clone, Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BlockLevelNode {
    InitVar(InitializedVariable),
    Block(ExpressionBlock),
    Function(BodiedFunction),
    Operation(Operation),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl SpanBounds for Body {
    fn bounds(&self) -> Span {
        match self {
            Body::Block(x) => x.bounds(),
            Body::Simplified { flow, expression } => flow.0 + expression.bounds(),
        }
    }
}

impl SpanBounds for BlockLevelNode {
    fn bounds(&self) -> Span {
        match self {
            BlockLevelNode::InitVar(x) => x.bounds(),
            BlockLevelNode::Block(x) => x.bounds(),
            BlockLevelNode::Function(x) => x.bounds(),
            BlockLevelNode::Operation(x) => x.bounds(),
        }
    }
}

impl SpanBounds for InitializedVariable {
    fn bounds(&self) -> Span {
        self.variable.bounds() + self.expression.bounds()
    }
}

impl SpanBounds for Variable {
    fn bounds(&self) -> Span {
        match self {
            Variable::Immutable {
                keyword,
                assigned_type,
                ..
            } => keyword.0 + assigned_type.as_ref().map(|it| it.1.location()),
            Variable::Mutable {
                keyword,
                assigned_type,
                ..
            } => keyword.0 + assigned_type.as_ref().map(|it| it.1.location()),
        }
    }
}
