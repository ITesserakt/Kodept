use crate::code_point::CodePoint;
use crate::structure::rlt::new_types::Keyword;
use crate::structure::rlt::{Body, Operation};
use crate::structure::Located;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct IfExpr {
    pub keyword: Keyword,
    pub condition: Operation,
    pub body: Body,
    pub elif: Box<[ElifExpr]>,
    pub el: Option<ElseExpr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct ElifExpr {
    pub keyword: Keyword,
    pub condition: Operation,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct ElseExpr {
    pub keyword: Keyword,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
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
