use crate::new_types::{Identifier, Keyword, Symbol};
use crate::prelude::{BodiedFunction, ExpressionBlock, Operation, Type};
use derive_more::From;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Body {
    Block(ExpressionBlock),
    Simplified {
        flow: Symbol,
        expression: BlockLevelNode,
    },
}

#[derive(Clone, Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BlockLevelNode {
    InitVar(InitializedVariable),
    Block(ExpressionBlock),
    Function(BodiedFunction),
    Operation(Operation),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Variable {
    Immutable {
        keyword: Keyword,
        id: Identifier,
        assigned_type: Option<(Symbol, Type)>,
    },
    Mutable {
        keyword: Keyword,
        id: Identifier,
        assigned_type: Option<(Symbol, Type)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct InitializedVariable {
    pub variable: Variable,
    pub equals: Symbol,
    pub expression: Operation,
}

impl Located for Variable {
    fn location(&self) -> CodePoint {
        match self {
            Variable::Immutable { id, .. } => id.location(),
            Variable::Mutable { id, .. } => id.location(),
        }
    }
}

impl Located for InitializedVariable {
    fn location(&self) -> CodePoint {
        self.variable.location()
    }
}

impl Located for Body {
    fn location(&self) -> CodePoint {
        match self {
            Body::Block(x) => x.location(),
            Body::Simplified { expression, .. } => expression.location(),
        }
    }
}

impl Located for BlockLevelNode {
    fn location(&self) -> CodePoint {
        match self {
            BlockLevelNode::InitVar(x) => x.location(),
            BlockLevelNode::Block(x) => x.location(),
            BlockLevelNode::Function(x) => x.location(),
            BlockLevelNode::Operation(x) => x.location(),
        }
    }
}

impl SpanBounds for Body {
    fn bounds(&self) -> Span {
        match self {
            Body::Block(x) => x.bounds(),
            Body::Simplified { flow, expression } => flow.0 + expression.bounds(),
        }
    }
}

impl SpanBounds for BlockLevelNode {
    fn bounds(&self) -> Span {
        match self {
            BlockLevelNode::InitVar(x) => x.bounds(),
            BlockLevelNode::Block(x) => x.bounds(),
            BlockLevelNode::Function(x) => x.bounds(),
            BlockLevelNode::Operation(x) => x.bounds(),
        }
    }
}

impl SpanBounds for InitializedVariable {
    fn bounds(&self) -> Span {
        self.variable.bounds() + self.expression.bounds()
    }
}

impl SpanBounds for Variable {
    fn bounds(&self) -> Span {
        match self {
            Variable::Immutable {
                keyword,
                assigned_type,
                id,
                ..
            } => keyword.0 + assigned_type.as_ref().map(|it| it.1.location()) + id.0,
            Variable::Mutable {
                keyword,
                assigned_type,
                id,
                ..
            } => keyword.0 + assigned_type.as_ref().map(|it| it.1.location()) + id.0,
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arb {
    use crate::block_level::{Body, InitializedVariable};
    use crate::new_types::{Enclosed, Identifier, Keyword, Symbol};
    use crate::prelude::{BlockLevelNode, BodiedFunction, ExpressionBlock, Operation, Parameter, Type};
    use proptest::collection::vec;
    use proptest::prelude::{any, Arbitrary, BoxedStrategy, Strategy};
    use proptest::prop_oneof;

    impl Arbitrary for BlockLevelNode {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            let leaf = prop_oneof![
                any::<InitializedVariable>().prop_map(BlockLevelNode::InitVar),
                any::<Operation>().prop_map(BlockLevelNode::Operation)
            ];

            leaf.prop_recursive(4, 20, 5, |inner| {
                prop_oneof![
                    (any::<Symbol>(), vec(inner.clone(), 0..10), any::<Symbol>()).prop_map(|it| {
                        BlockLevelNode::Block(ExpressionBlock {
                            lbrace: it.0,
                            rbrace: it.2,
                            expression: it.1.into_boxed_slice(),
                        })
                    }),
                    (
                        any::<Identifier>(),
                        any::<Keyword>(),
                        any::<Option<Enclosed<Box<[Parameter]>>>>(),
                        any::<Option<(Symbol, Type)>>(),
                        any::<Symbol>(),
                        inner
                    )
                        .prop_map(|it| {
                            BlockLevelNode::Function(BodiedFunction {
                                id: it.0,
                                keyword: it.1,
                                params: it.2,
                                return_type: it.3,
                                body: Box::new(Body::Simplified {
                                    flow: it.4,
                                    expression: it.5,
                                }),
                            })
                        })
                ]
            })
            .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}
