use crate::new_types::Keyword;
use crate::prelude::{Body, Operation};
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfExpr {
    pub keyword: Keyword,
    pub condition: Operation,
    pub body: Body,
    pub elif: Box<[ElifExpr]>,
    pub el: Option<ElseExpr>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElifExpr {
    pub keyword: Keyword,
    pub condition: Operation,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl SpanBounds for IfExpr {
    fn bounds(&self) -> Span {
        self.keyword.0
            + self.body.bounds()
            + self.elif.last().map(|it| it.bounds())
            + self.el.as_ref().map(|it| it.bounds())
    }
}

impl SpanBounds for ElifExpr {
    fn bounds(&self) -> Span {
        self.keyword.0 + self.body.bounds()
    }
}

impl SpanBounds for ElseExpr {
    fn bounds(&self) -> Span {
        self.keyword.0 + self.body.bounds()
    }
}
