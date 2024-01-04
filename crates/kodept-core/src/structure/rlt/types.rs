use derive_more::From;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::new_types::*;

#[derive(Debug, Clone, PartialEq, From)]
pub enum Type {
    Reference(TypeName),
    #[from(ignore)]
    Tuple(Enclosed<Box<[Type]>>),
    #[from(ignore)]
    Union(Enclosed<Box<[Type]>>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct TypedParameter {
    pub id: Identifier,
    pub parameter_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct UntypedParameter {
    pub id: Identifier,
}

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Parameter {
    Typed(TypedParameter),
    Untyped(UntypedParameter),
}

#[cfg(feature = "size-of")]
impl SizeOf for Type {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            Type::Reference(x) => x.size_of_children(context),
            Type::Tuple(x) => x.size_of_children(context),
            Type::Union(x) => x.size_of_children(context),
        }
    }
}

impl Located for Type {
    fn location(&self) -> CodePoint {
        match self {
            Type::Reference(x) => x.location(),
            Type::Tuple(x) => x.left.location(),
            Type::Union(x) => x.left.location(),
        }
    }
}

impl Located for TypedParameter {
    fn location(&self) -> CodePoint {
        self.id.location()
    }
}

impl Located for UntypedParameter {
    fn location(&self) -> CodePoint {
        self.id.location()
    }
}

impl Located for Parameter {
    fn location(&self) -> CodePoint {
        match self {
            Parameter::Typed(x) => x.location(),
            Parameter::Untyped(x) => x.location(),
        }
    }
}
