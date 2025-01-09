use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use crate::{BlockLevelNode, IfExpr, Literal, Parameter, Term};
use crate::new_types::{BinaryOperationSymbol, Enclosed, Symbol, UnaryOperationSymbol};

#[derive(Debug, Clone, PartialEq)]
pub struct Application {
    pub expr: Operation,
    pub params: Option<Enclosed<Box<[Operation]>>>,
}

#[derive(Debug, Clone, PartialEq)]
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
pub enum Expression {
    Lambda(Lambda),
    Term(Term),
    Literal(Literal),
    If(Box<IfExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    pub binds: Enclosed<Box<[Parameter]>>,
    pub flow: Symbol,
    pub expr: Box<Operation>,
}

#[derive(Debug, Clone, PartialEq)]
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
