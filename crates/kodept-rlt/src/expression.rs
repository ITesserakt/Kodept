use crate::new_types::{BinaryOperationSymbol, Enclosed, Symbol, UnaryOperationSymbol};
use crate::prelude::{BlockLevelNode, IfExpr, Literal, Parameter, Term};
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Application {
    pub expr: Operation,
    pub params: Option<Enclosed<Box<[Operation]>>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Operation {
    Block(ExpressionBlock),
    Access {
        left: Box<Operation>,
        dot: Symbol,
        right: Box<Operation>,
    },
    Unary {
        operator: UnaryOperationSymbol,
        expr: Box<Operation>,
    },
    Binary {
        left: Box<Operation>,
        operation: BinaryOperationSymbol,
        right: Box<Operation>,
    },
    Application(Box<Application>),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Expression {
    Lambda(Lambda),
    Term(Term),
    Literal(Literal),
    If(Box<IfExpr>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lambda {
    pub binds: Enclosed<Box<[Parameter]>>,
    pub flow: Symbol,
    pub expr: Box<Operation>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionBlock {
    pub lbrace: Symbol,
    pub expression: Box<[BlockLevelNode]>,
    pub rbrace: Symbol,
}

impl Located for Application {
    fn location(&self) -> CodePoint {
        self.params
            .as_ref()
            .map_or(self.expr.location(), |it| it.left.location())
    }
}

impl Located for Operation {
    fn location(&self) -> CodePoint {
        match self {
            Operation::Block(x) => x.location(),
            Operation::Access { dot, .. } => dot.location(),
            Operation::Unary { operator, .. } => operator.location(),
            Operation::Binary { operation, .. } => operation.location(),
            Operation::Application(x) => x.location(),
            Operation::Expression(x) => x.location(),
        }
    }
}

impl Located for Expression {
    fn location(&self) -> CodePoint {
        match self {
            Expression::Lambda(x) => x.location(),
            Expression::Term(x) => x.location(),
            Expression::Literal(x) => x.location(),
            Expression::If(x) => x.location(),
        }
    }
}

impl Located for Lambda {
    fn location(&self) -> CodePoint {
        self.flow.location()
    }
}

impl Located for ExpressionBlock {
    fn location(&self) -> CodePoint {
        self.lbrace.location()
    }
}

impl SpanBounds for ExpressionBlock {
    fn bounds(&self) -> Span {
        self.lbrace.0 + self.rbrace.0
    }
}

impl SpanBounds for Expression {
    fn bounds(&self) -> Span {
        match self {
            Expression::Lambda(x) => x.bounds(),
            Expression::Term(x) => x.bounds(),
            Expression::Literal(x) => x.bounds(),
            Expression::If(x) => x.bounds(),
        }
    }
}

impl SpanBounds for Lambda {
    fn bounds(&self) -> Span {
        self.binds.left.0 + self.expr.bounds()
    }
}

impl SpanBounds for Operation {
    fn bounds(&self) -> Span {
        match self {
            Operation::Block(x) => x.bounds(),
            Operation::Access { left, right, .. } => left.bounds() + right.bounds(),
            Operation::Unary { operator, expr } => operator.location() + expr.bounds(),
            Operation::Binary { left, right, .. } => left.bounds() + right.bounds(),
            Operation::Application(x) => x.bounds(),
            Operation::Expression(x) => x.bounds(),
        }
    }
}

impl SpanBounds for Application {
    fn bounds(&self) -> Span {
        self.expr.bounds() + self.params.as_ref().map(|it| it.left.0 + it.right.0)
    }
}
