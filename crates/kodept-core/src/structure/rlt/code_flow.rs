use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::{Body, Operation};
use crate::structure::rlt::new_types::Keyword;

#[derive(Clone, Debug, PartialEq)]
pub struct IfExpr {
    pub keyword: Keyword,
    pub condition: Operation,
    pub body: Body,
    pub elif: Box<[ElifExpr]>,
    pub el: Option<ElseExpr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElifExpr {
    pub keyword: Keyword,
    pub condition: Operation,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElseExpr {
    pub keyword: Keyword,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CodeFlow {
    If(IfExpr),
}

impl Located for IfExpr {
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl Located for ElifExpr {
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl Located for ElseExpr {
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl Located for CodeFlow {
    fn location(&self) -> CodePoint {
        match self {
            CodeFlow::If(x) => x.location(),
        }
    }
}
