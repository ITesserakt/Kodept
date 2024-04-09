use derive_more::{From, Into};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use BinaryExpressionKind::*;
use UnaryExpressionKind::*;

use crate::graph::tags::*;
use crate::graph::NodeId;
use crate::graph::{GenericASTNode, NodeUnion};
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::traits::{Linker, PopulateTree};
use crate::{node, wrapper, BlockLevel, IfExpression, Literal, Term, UntypedParameter};

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Operation {
        application(Application) = GenericASTNode::Application(x) => Some(x),
        access(Access) = GenericASTNode::Access(x) => Some(x),
        unary(Unary) = GenericASTNode::Unary(x) => Some(x),
        binary(Binary) = GenericASTNode::Binary(x) => Some(x),
        block(ExpressionBlock) = GenericASTNode::ExpressionBlock(x) => Some(x),
        expression(Expression) = n if Expression::contains(n) => n.try_into().ok(),
    }
}

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Expression {
        lambda(Lambda) = GenericASTNode::Lambda(x) => Some(x),
        if(IfExpression) = GenericASTNode::If(x) => Some(x),
        literal(Literal) = n if Literal::contains(n) => n.try_into().ok(),
        term(Term) = n if Term::contains(n) => n.try_into().ok(),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Application {;
        pub expr: Identity<Operation> as PRIMARY,
        pub params: Vec<Operation> as SECONDARY,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Access {;
        pub left: Identity<Operation> as LEFT,
        pub right: Identity<Operation> as RIGHT,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Unary {
        pub kind: UnaryExpressionKind,;
        pub expr: Identity<Operation>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Binary {
        pub kind: BinaryExpressionKind,;
        pub left: Identity<Operation> as LEFT,
        pub right: Identity<Operation> as RIGHT,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Lambda {;
        // binds somehow wrapped in operation causing expr to fail => tags required
        pub binds: Vec<UntypedParameter> as PRIMARY,
        pub expr: Identity<Operation> as SECONDARY,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ExpressionBlock {;
        pub items: Vec<BlockLevel>,
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum UnaryExpressionKind {
    Neg,
    Not,
    Inv,
    Plus,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BinaryExpressionKind {
    Pow,
    Mul,
    Add,
    ComplexComparison,
    CompoundComparison,
    Comparison,
    Bit,
    Logic,
}

impl PopulateTree for rlt::ExpressionBlock {
    type Output = ExpressionBlock;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(ExpressionBlock {
                id: Default::default(),
            })
            .with_children_from(self.expression.as_ref(), context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Operation {
    type Output = Operation;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Operation::Block(x) => x.convert(builder, context).cast(),
            rlt::Operation::Access { left, right, .. } => builder
                .add_node(Access {
                    id: Default::default(),
                })
                .with_children_from::<LEFT, _>([left.as_ref()], context)
                .with_children_from::<RIGHT, _>([right.as_ref()], context)
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Operation::TopUnary { operator, expr } => builder
                .add_node(Unary {
                    kind: match operator {
                        UnaryOperationSymbol::Neg(_) => Neg,
                        UnaryOperationSymbol::Not(_) => Not,
                        UnaryOperationSymbol::Inv(_) => Inv,
                        UnaryOperationSymbol::Plus(_) => Plus,
                    },
                    id: Default::default(),
                })
                .with_children_from([expr.as_ref()], context)
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Operation::Binary {
                left,
                operation,
                right,
            } => builder
                .add_node(Binary {
                    kind: match operation {
                        BinaryOperationSymbol::Pow(_) => Pow,
                        BinaryOperationSymbol::Mul(_) => Mul,
                        BinaryOperationSymbol::Add(_) => Add,
                        BinaryOperationSymbol::ComplexComparison(_) => ComplexComparison,
                        BinaryOperationSymbol::CompoundComparison(_) => CompoundComparison,
                        BinaryOperationSymbol::Comparison(_) => Comparison,
                        BinaryOperationSymbol::Bit(_) => Bit,
                        BinaryOperationSymbol::Logic(_) => Logic,
                    },
                    id: Default::default(),
                })
                .with_children_from::<LEFT, _>([left.as_ref()], context)
                .with_children_from::<RIGHT, _>([right.as_ref()], context)
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Operation::Application(x) => x.convert(builder, context).cast(),
            rlt::Operation::Expression(x) => x.convert(builder, context).cast(),
        }
    }
}

impl PopulateTree for rlt::Application {
    type Output = Application;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(Application {
                id: Default::default(),
            })
            .with_children_from::<PRIMARY, _>([&self.expr], context)
            .with_children_from::<SECONDARY, _>(
                self.params
                    .as_ref()
                    .map_or([].as_slice(), |x| x.inner.as_ref()),
                context,
            )
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Expression {
    type Output = Expression;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Expression::Lambda { binds, expr, .. } => builder
                .add_node(Lambda {
                    id: Default::default(),
                })
                .with_children_from(binds.as_ref(), context)
                .with_children_from([expr.as_ref()], context)
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Expression::Term(x) => x.convert(builder, context).cast(),
            rlt::Expression::Literal(x) => x.convert(builder, context).cast(),
            rlt::Expression::If(x) => x.convert(builder, context).cast(),
        }
    }
}

impl Application {
    pub fn new() -> Self {
        Application {
            id: Default::default(),
        }
    }
}
