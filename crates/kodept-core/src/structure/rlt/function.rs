use derive_more::From;

use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::block_level::Body;
use crate::structure::rlt::new_types::*;
use crate::structure::rlt::types::{Parameter, Type, TypedParameter};

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
