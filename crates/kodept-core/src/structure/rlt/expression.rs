use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::{IfExpr, Literal, Parameter, Term};
use crate::structure::rlt::block_level::BlockLevelNode;
use crate::structure::rlt::new_types::*;

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
    TopUnary {
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
    Lambda {
        keyword: Keyword,
        binds: Box<[Parameter]>,
        flow: Symbol,
        expr: Box<Operation>,
    },
    Term(Term),
    Literal(Literal),
    If(Box<IfExpr>),
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
            Operation::TopUnary { operator, .. } => operator.location(),
            Operation::Binary { operation, .. } => operation.location(),
            Operation::Application(x) => x.location(),
            Operation::Expression(x) => x.location(),
        }
    }
}

impl Located for Expression {
    fn location(&self) -> CodePoint {
        match self {
            Expression::Lambda { flow, .. } => flow.location(),
            Expression::Term(x) => x.location(),
            Expression::Literal(x) => x.location(),
            Expression::If(x) => x.location(),
        }
    }
}

impl Located for ExpressionBlock {
    fn location(&self) -> CodePoint {
        self.lbrace.location()
    }
}
