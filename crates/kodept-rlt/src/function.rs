use crate::new_types::{Enclosed, Identifier, Keyword, Symbol};
use crate::prelude::{Body, Parameter, Type, TypedParameter};
use derive_more::From;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct BodiedFunction {
    pub keyword: Keyword,
    pub id: Identifier,
    pub params: Option<Enclosed<Box<[Parameter]>>>,
    pub return_type: Option<(Symbol, Type)>,
    pub body: Box<Body>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AbstractFunction {
    pub keyword: Keyword,
    pub id: Identifier,
    pub params: Option<Enclosed<Box<[TypedParameter]>>>,
    pub return_type: Option<(Symbol, Type)>,
}

#[derive(Clone, Debug, PartialEq, From)]
pub enum Function {
    Abstract(AbstractFunction),
    Bodied(BodiedFunction),
}

impl Located for BodiedFunction {
    #[inline]
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl SpanBounds for BodiedFunction {
    #[inline]
    fn bounds(&self) -> Span {
        self.keyword.0 + self.body.bounds()
    }
}

impl Located for AbstractFunction {
    #[inline]
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl Located for Function {
    #[inline]
    fn location(&self) -> CodePoint {
        match self {
            Function::Abstract(x) => x.location(),
            Function::Bodied(x) => x.location(),
        }
    }
}
