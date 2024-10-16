use derive_more::From;

use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::function::BodiedFunction;
use crate::structure::rlt::new_types::*;
use crate::structure::rlt::types::TypedParameter;

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub keyword: Keyword,
    pub id: TypeName,
    pub parameters: Option<Enclosed<Box<[TypedParameter]>>>,
    pub body: Option<Enclosed<Box<[BodiedFunction]>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Enum {
    Stack {
        keyword: Keyword,
        id: TypeName,
        contents: Option<Enclosed<Box<[TypeName]>>>,
    },
    Heap {
        keyword: Keyword,
        id: TypeName,
        contents: Option<Enclosed<Box<[TypeName]>>>,
    },
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum TopLevelNode {
    Enum(Enum),
    Struct(Struct),
    BodiedFunction(BodiedFunction),
}

impl Located for Struct {
    fn location(&self) -> CodePoint {
        self.keyword.location()
    }
}

impl Located for Enum {
    fn location(&self) -> CodePoint {
        match self {
            Enum::Stack { keyword, .. } => keyword.location(),
            Enum::Heap { keyword, .. } => keyword.location(),
        }
    }
}

impl Located for TopLevelNode {
    fn location(&self) -> CodePoint {
        match self {
            TopLevelNode::Enum(x) => x.location(),
            TopLevelNode::Struct(x) => x.location(),
            TopLevelNode::BodiedFunction(x) => x.location(),
        }
    }
}

impl Enum {
    pub fn id(&self) -> &TypeName {
        match self {
            Enum::Stack { id, .. } => id,
            Enum::Heap { id, .. } => id,
        }
    }
}
