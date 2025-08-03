use crate::new_types::{Enclosed, Identifier, TypeName};
use crate::prelude::Context;
use derive_more::From;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    ContextualReference(Context, TypeName),
    Reference(TypeName),
    Tuple(Tuple),
}

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct Tuple(pub Enclosed<Box<[Type]>>);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct TypedParameter {
    pub id: Identifier,
    pub parameter_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct UntypedParameter {
    pub id: Identifier,
}

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Parameter {
    Typed(TypedParameter),
    Untyped(UntypedParameter),
}

impl Located for Type {
    fn location(&self) -> CodePoint {
        match self {
            Type::ContextualReference(context, ty) => {
                let (is_global, unfolded) = context.unfold();
                let first = unfolded.first().map_or(ty.location(), |it| it.location());
                let last = ty.location();
                let length = (first + last).length;
                if is_global.is_some() {
                    CodePoint::new(length + 2, first.offset - 2)
                } else {
                    CodePoint::new(length, first.offset)
                }
            }
            Type::Reference(x) => x.location(),
            Type::Tuple(x) => x.location(),
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

impl Located for Tuple {
    fn location(&self) -> CodePoint {
        self.0.left.location()
    }
}

impl SpanBounds for Tuple {
    fn bounds(&self) -> Span {
        self.0.left.bounds() + self.0.right.bounds()
    }
}

impl SpanBounds for Type {
    fn bounds(&self) -> Span {
        match self {
            Type::Reference(x) => x.0.into(),
            Type::Tuple(x) => x.0.left.0 + x.0.right.0,
            Type::ContextualReference(_, _) => self.location().into(),
        }
    }
}

impl SpanBounds for TypedParameter {
    fn bounds(&self) -> Span {
        self.id.0 + self.parameter_type.bounds()
    }
}

impl SpanBounds for UntypedParameter {
    fn bounds(&self) -> Span {
        self.id.0.into()
    }
}

#[cfg(feature = "arbitrary")]
mod arb {
    use crate::new_types::{Enclosed, Symbol, TypeName};
    use crate::types::{Tuple, Type};
    use proptest::collection::vec;
    use proptest::prelude::{any, Arbitrary, BoxedStrategy, Strategy};

    impl Arbitrary for Type {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<TypeName>()
                .prop_map(Type::Reference)
                .prop_recursive(4, 20, 5, |inner| {
                    (any::<Symbol>(), vec(inner, 0..10), any::<Symbol>())
                        .prop_map(|it| Type::Tuple(Tuple(Enclosed::from(it))))
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Type>;
    }
}
