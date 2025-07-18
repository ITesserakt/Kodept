use crate::new_types::{BinaryOperationSymbol, Enclosed, Symbol, UnaryOperationSymbol};
use crate::prelude::{BlockLevelNode, IfExpr, Literal, Parameter, Term};
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Application {
    pub expr: Operation,
    pub params: Option<Enclosed<Box<[Operation]>>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Operation {
    Block(ExpressionBlock),
    Access {
        left: Box<Operation>,
        dot: Symbol,
        right: Box<Operation>,
    },
    Unary {
        operator: UnaryOperationSymbol,
        expr: Box<Operation>,
    },
    Binary {
        left: Box<Operation>,
        operation: BinaryOperationSymbol,
        right: Box<Operation>,
    },
    Application(Box<Application>),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Expression {
    Lambda(Lambda),
    Term(Term),
    Literal(Literal),
    If(Box<IfExpr>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lambda {
    pub binds: Enclosed<Box<[Parameter]>>,
    pub flow: Symbol,
    pub expr: Box<Operation>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionBlock {
    pub lbrace: Symbol,
    pub expression: Box<[BlockLevelNode]>,
    pub rbrace: Symbol,
}

impl Located for Application {
    fn location(&self) -> CodePoint {
        self.params
            .as_ref()
            .map_or(self.expr.location(), |it| it.left.location())
    }
}

impl Located for Operation {
    fn location(&self) -> CodePoint {
        match self {
            Operation::Block(x) => x.location(),
            Operation::Access { dot, .. } => dot.location(),
            Operation::Unary { operator, .. } => operator.location(),
            Operation::Binary { operation, .. } => operation.location(),
            Operation::Application(x) => x.location(),
            Operation::Expression(x) => x.location(),
        }
    }
}

impl Located for Expression {
    fn location(&self) -> CodePoint {
        match self {
            Expression::Lambda(x) => x.location(),
            Expression::Term(x) => x.location(),
            Expression::Literal(x) => x.location(),
            Expression::If(x) => x.location(),
        }
    }
}

impl Located for Lambda {
    fn location(&self) -> CodePoint {
        self.flow.location()
    }
}

impl Located for ExpressionBlock {
    fn location(&self) -> CodePoint {
        self.lbrace.location()
    }
}

impl SpanBounds for ExpressionBlock {
    fn bounds(&self) -> Span {
        self.lbrace.0 + self.rbrace.0
    }
}

impl SpanBounds for Expression {
    fn bounds(&self) -> Span {
        match self {
            Expression::Lambda(x) => x.bounds(),
            Expression::Term(x) => x.bounds(),
            Expression::Literal(x) => x.bounds(),
            Expression::If(x) => x.bounds(),
        }
    }
}

impl SpanBounds for Lambda {
    fn bounds(&self) -> Span {
        self.binds.left.0 + self.expr.bounds()
    }
}

impl SpanBounds for Operation {
    fn bounds(&self) -> Span {
        match self {
            Operation::Block(x) => x.bounds(),
            Operation::Access { left, right, .. } => left.bounds() + right.bounds(),
            Operation::Unary { operator, expr } => operator.location() + expr.bounds(),
            Operation::Binary { left, right, .. } => left.bounds() + right.bounds(),
            Operation::Application(x) => x.bounds(),
            Operation::Expression(x) => x.bounds(),
        }
    }
}

impl SpanBounds for Application {
    fn bounds(&self) -> Span {
        self.expr.bounds() + self.params.as_ref().map(|it| it.left.0 + it.right.0)
    }
}

#[cfg(feature = "arbitrary")]
mod arb {
    use crate::block_level::BlockLevelNode;
    use crate::expression::{Application, ExpressionBlock, Operation};
    use crate::literal::Literal;
    use crate::new_types::{BinaryOperationSymbol, Enclosed, Symbol, UnaryOperationSymbol};
    use crate::prelude::{Expression, Lambda};
    use crate::term::Term;
    use crate::types::Parameter;
    use kodept_core::code_point::CodePoint;
    use proptest::collection::vec;
    use proptest::prelude::{any, Arbitrary, BoxedStrategy, Strategy};
    use proptest::prop_oneof;

    impl Arbitrary for Operation {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            let leaf = prop_oneof![
                any::<Term>().prop_map(|it| Operation::Expression(Expression::Term(it))),
                any::<CodePoint>()
                    .prop_map(|it| Operation::Expression(Expression::Literal(Literal::String(it)))),
            ];
            leaf.prop_recursive(5, 20, 5, |inner| {
                prop_oneof![
                    (inner.clone(), any::<Symbol>(), inner.clone()).prop_map(|it| {
                        Operation::Access {
                            left: Box::new(it.0),
                            dot: it.1,
                            right: Box::new(it.2),
                        }
                    }),
                    (any::<UnaryOperationSymbol>(), inner.clone()).prop_map(|it| {
                        Operation::Unary {
                            operator: it.0,
                            expr: Box::new(it.1),
                        }
                    }),
                    (inner.clone(), any::<BinaryOperationSymbol>(), inner.clone()).prop_map(|it| {
                        Operation::Binary {
                            left: Box::new(it.0),
                            operation: it.1,
                            right: Box::new(it.2),
                        }
                    }),
                    (
                        inner.clone(),
                        proptest::option::of((
                            any::<Symbol>(),
                            vec(inner.clone(), 0..5),
                            any::<Symbol>()
                        ))
                    )
                        .prop_map(|it| {
                            Operation::Application(Box::new(Application {
                                expr: it.0,
                                params: it.1.map(Enclosed::from),
                            }))
                        }),
                    (
                        inner.clone(),
                        any::<Symbol>(),
                        (
                            any::<Symbol>(),
                            vec(any::<Parameter>(), 0..10),
                            any::<Symbol>()
                        )
                    )
                        .prop_map(|it| {
                            Operation::Expression(Expression::Lambda(Lambda {
                                flow: it.1,
                                expr: Box::new(it.0),
                                binds: it.2.into(),
                            }))
                        }),
                    (any::<Symbol>(), vec(inner.clone(), 0..10), any::<Symbol>()).prop_map(|it| {
                        Operation::Block(ExpressionBlock {
                            lbrace: it.0,
                            rbrace: it.2,
                            expression: it.1.into_iter().map(BlockLevelNode::Operation).collect(),
                        })
                    }),
                    // TODO: generate ifs
                ]
            })
            .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }

    impl Arbitrary for ExpressionBlock {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            (
                any::<Symbol>(),
                vec(any::<BlockLevelNode>(), 0..10),
                any::<Symbol>(),
            )
                .prop_map(|it| ExpressionBlock {
                    lbrace: it.0,
                    rbrace: it.2,
                    expression: it.1.into_boxed_slice(),
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(all(test, feature = "arbitrary"))]
mod tests {
    use crate::expression::Operation;
    use proptest::proptest;

    proptest! {
        #[test]
        fn test_generation(_: Operation) {}
    }
}
