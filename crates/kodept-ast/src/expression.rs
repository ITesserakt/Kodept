use crate::graph::generic_node::GenericASTNode;
use crate::graph::traits::PopulateTree;
use crate::graph::{Identity, SyntaxTree};
use crate::node_id::NodeId;
use crate::traits::Linker;
use crate::{
    impl_identifiable_2, with_children, wrapper, BlockLevel, IfExpression, Literal, Reference, Term,
};
use derive_more::{Deref, DerefMut, From};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use BinaryExpressionKind::*;
use UnaryExpressionKind::*;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Application {
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Access {
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Unary {
    pub kind: UnaryExpressionKind,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Binary {
    pub kind: BinaryExpressionKind,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Operation {
    Application(Box<Application>),
    Access(Box<Access>),
    Unary(Box<Unary>),
    Binary(Box<Binary>),
    Expression(Box<Expression>),
    Block(Box<ExpressionBlock>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Lambda {
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Expression {
    Lambda(Lambda),
    Term(Term),
    Literal(Literal),
    If(IfExpression),
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

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExpressionBlock {
    id: NodeId<Self>,
}

#[cfg(feature = "size-of")]
impl SizeOf for Operation {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            Operation::Application(x) => x.size_of_children(context),
            Operation::Access(x) => x.size_of_children(context),
            Operation::Unary(x) => x.size_of_children(context),
            Operation::Binary(x) => x.size_of_children(context),
            Operation::Expression(x) => x.size_of_children(context),
            Operation::Block(x) => x.size_of_children(context),
        }
    }
}

impl_identifiable_2! {
    Application,
    Access,
    Unary,
    Binary,
    Lambda,
    ExpressionBlock
}

with_children!(ExpressionBlock => {
    pub items: Vec<BlockLevel>
});

with_children!(Lambda => {
    pub binds: Vec<Reference>
    pub expr: Identity<Operation>
});

with_children!(Unary => {
    pub expr: Identity<Operation>
});

wrapper! {
    #[derive(Deref, DerefMut)]
    pub wrapper AOperation(Operation);
}

with_children!(Application => {
    pub expr: Identity<AOperation>
    pub params: Vec<Operation>
});

wrapper! {
    #[derive(Deref, DerefMut)]
    pub wrapper LeftOperation(Operation);
}
wrapper! {
    #[derive(Deref, DerefMut)]
    pub wrapper RightOperation(Operation);
}

with_children!(Access => {
    _syntethic: Vec<Operation>
    pub left: Identity<LeftOperation>
    pub right: Identity<RightOperation>
});

with_children!(Binary => {
    _syntethic: Vec<Operation>
    pub left: Identity<LeftOperation>
    pub right: Identity<RightOperation>
});

impl PopulateTree for rlt::ExpressionBlock {
    type Output = ExpressionBlock;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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
