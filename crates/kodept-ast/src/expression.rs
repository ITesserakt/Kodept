use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use BinaryExpressionKind::*;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_core::structure::span::CodeHolder;
use UnaryExpressionKind::*;

use crate::{BlockLevel, IfExpression, Literal, node, Reference, Term, wrapper};
use crate::graph::{GenericASTNode, NodeUnion};
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::graph::NodeId;
use crate::traits::{Linker, PopulateTree};
use crate::wrappers::{AOperation, LeftOperation, RightOperation};

pub mod wrappers {
    use derive_more::{Deref, DerefMut};

    use crate::{graph::GenericASTNode, Operation, wrapper};

    wrapper! {
        #[derive(Deref, DerefMut)]
        pub wrapper AOperation(Operation);
    }
    wrapper! {
        #[derive(Deref, DerefMut)]
        pub wrapper LeftOperation(Operation);
    }
    wrapper! {
        #[derive(Deref, DerefMut)]
        pub wrapper RightOperation(Operation);
    }
}

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
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Application {;
        pub expr: Identity<AOperation>,
        pub params: Vec<Operation>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Access {;
        _syntethic: Vec<Operation>,
        pub left: Identity<LeftOperation>,
        pub right: Identity<RightOperation>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Unary {
        pub kind: UnaryExpressionKind,;
        pub expr: Identity<Operation>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Binary {
        pub kind: BinaryExpressionKind,;
        _syntethic: Vec<Operation>,
        pub left: Identity<LeftOperation>,
        pub right: Identity<RightOperation>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Lambda {;
        pub binds: Vec<Reference>,
        pub expr: Identity<Operation>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ExpressionBlock {;
        pub items: Vec<BlockLevel>,
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum UnaryExpressionKind {
    Neg,
    Not,
    Inv,
    Plus,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
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

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
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

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Operation::Block(x) => x.convert(builder, context).cast(),
            rlt::Operation::Access { left, right, .. } => builder
                .add_node(Access {
                    id: Default::default(),
                })
                .with_children_from([left.as_ref(), right.as_ref()], context)
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
                .with_children_from([left.as_ref(), right.as_ref()], context)
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

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(Application {
                id: Default::default(),
            })
            .with_children_from([&self.expr], context)
            .with_children_from(
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

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
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
