use derive_more::From;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

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
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct AbstractFunction {
    pub keyword: Keyword,
    pub id: Identifier,
    pub params: Option<Enclosed<Box<[TypedParameter]>>>,
    pub return_type: Option<(Symbol, Type)>,
}

#[derive(Clone, Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Function {
    Abstract(AbstractFunction),
    Bodied(BodiedFunction),
}

#[cfg(feature = "size-of")]
impl ::size_of::SizeOf for BodiedFunction {
    fn size_of_children(&self, context: &mut size_of::Context) {
        self.keyword.size_of_children(context);
        self.id.size_of_children(context);
        self.params.size_of_children(context);
        self.return_type.size_of_children(context);
        self.body.size_of_children(context);
    }
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
