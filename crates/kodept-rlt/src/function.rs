use derive_more::From;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;

use crate::new_types::*;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
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
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl Located for AbstractFunction {
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl Located for Function {
    fn location(&self) -> CodePoint {
        match self {
            Function::Abstract(x) => x.location(),
            Function::Bodied(x) => x.location(),
        }
    }
}
