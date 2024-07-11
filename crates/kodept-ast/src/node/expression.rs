#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use BinaryExpressionKind::*;
use kodept_core::structure::{rlt};
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_core::structure::span::CodeHolder;
use UnaryExpressionKind::*;

use crate::{BlockLevel, CodeFlow, Lit, node, node_sub_enum, Param, Term};
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::graph::NodeId;
use crate::graph::tags::*;
use crate::traits::{Linker, PopulateTree};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum Operation {
        Appl(Appl),
        Acc(Acc),
        Unary(UnExpr),
        Binary(BinExpr),
        Block(Exprs),
        Expr(forward Expression),
    }
}

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum Expression {
        Lambda(Lambda),
        CodeFlow(forward CodeFlow),
        Lit(forward Lit),
        Term(forward Term)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Appl {;
        pub expr: Identity<Operation> as PRIMARY,
        pub params: Vec<Operation> as SECONDARY,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Acc {;
        pub left: Identity<Operation> as LEFT,
        pub right: Identity<Operation> as RIGHT,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct UnExpr {
        pub kind: UnaryExpressionKind,;
        pub expr: Identity<Operation>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct BinExpr {
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
        pub binds: Vec<Param> as PRIMARY,
        pub expr: Identity<Operation> as SECONDARY,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Exprs {;
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
pub enum ComparisonKind {
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum EqKind {
    Eq,
    NEq,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum LogicKind {
    Disj,
    Conj,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BitKind {
    Or,
    And,
    Xor,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum MathKind {
    Add,
    Sub,
    Mul,
    Pow,
    Div,
    Mod,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BinaryExpressionKind {
    Math(MathKind),
    Cmp(ComparisonKind),
    Eq(EqKind),
    Bit(BitKind),
    Logic(LogicKind),
    ComplexComparison,
}

impl PopulateTree for rlt::ExpressionBlock {
    type Output = Exprs;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(Exprs::uninit())
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
            rlt::Operation::Access { left, right, .. } => {
                build_access(self, builder, context, left, right)
            }
            rlt::Operation::TopUnary { operator, expr } => {
                build_unary(self, builder, context, operator, expr)
            }
            rlt::Operation::Binary {
                left,
                operation,
                right,
            } => build_binary(self, builder, context, left, operation, right),
            rlt::Operation::Application(x) => x.convert(builder, context).cast(),
            rlt::Operation::Expression(x) => x.convert(builder, context).cast(),
        }
    }
}

fn build_binary(
    node: &rlt::Operation,
    builder: &mut SyntaxTreeBuilder,
    context: &mut (impl Linker + CodeHolder + Sized),
    left: &Box<rlt::Operation>,
    operation: &BinaryOperationSymbol,
    right: &Box<rlt::Operation>,
) -> NodeId<Operation> {
    let binding = context.get_chunk_located(operation);
    let op_text = binding.as_ref();
    
    builder
        .add_node(BinExpr::uninit(match (operation, op_text) {
            (BinaryOperationSymbol::Pow(_), _) => Math(MathKind::Pow),
            (BinaryOperationSymbol::Mul(_), "*") => Math(MathKind::Mul),
            (BinaryOperationSymbol::Mul(_), "/") => Math(MathKind::Div),
            (BinaryOperationSymbol::Mul(_), "%") => Math(MathKind::Mod),
            (BinaryOperationSymbol::Add(_), "+") => Math(MathKind::Add),
            (BinaryOperationSymbol::Add(_), "-") => Math(MathKind::Sub),
            (BinaryOperationSymbol::ComplexComparison(_), _) => ComplexComparison,
            (BinaryOperationSymbol::CompoundComparison(_), "<=") => Cmp(ComparisonKind::LessEq),
            (BinaryOperationSymbol::CompoundComparison(_), ">=") => Cmp(ComparisonKind::GreaterEq),
            (BinaryOperationSymbol::CompoundComparison(_), "!=") => Eq(EqKind::NEq),
            (BinaryOperationSymbol::CompoundComparison(_), "==") => Eq(EqKind::Eq),
            (BinaryOperationSymbol::Comparison(_), "<") => Cmp(ComparisonKind::Less),
            (BinaryOperationSymbol::Comparison(_), ">") => Cmp(ComparisonKind::Greater),
            (BinaryOperationSymbol::Bit(_), "|") => Bit(BitKind::Or),
            (BinaryOperationSymbol::Bit(_), "&") => Bit(BitKind::And),
            (BinaryOperationSymbol::Bit(_), "^") => Bit(BitKind::Xor),
            (BinaryOperationSymbol::Logic(_), "||") => Logic(LogicKind::Disj),
            (BinaryOperationSymbol::Logic(_), "&&") => Logic(LogicKind::Conj),
            
            (BinaryOperationSymbol::Mul(_), x) => panic!("Unknown mul operator found: {x}"),
            (BinaryOperationSymbol::Add(_), x) => panic!("Unknown add operator found: {x}"),
            (BinaryOperationSymbol::CompoundComparison(_), x) => panic!("Unknown cmp operator found: {x}"),
            (BinaryOperationSymbol::Comparison(_), x) => panic!("Unknown cmp operator found: {x}"),
            (BinaryOperationSymbol::Bit(_), x) => panic!("Unknown bit operator found: {x}"),
            (BinaryOperationSymbol::Logic(_), x) => panic!("Unknown logic operator found: {x}"),
        }))
        .with_children_from::<LEFT, _>([left.as_ref()], context)
        .with_children_from::<RIGHT, _>([right.as_ref()], context)
        .with_rlt(context, node)
        .id()
        .cast()
}

fn build_unary(
    node: &rlt::Operation,
    builder: &mut SyntaxTreeBuilder,
    context: &mut (impl Linker + CodeHolder + Sized),
    operator: &UnaryOperationSymbol,
    expr: &Box<rlt::Operation>,
) -> NodeId<Operation> {
    builder
        .add_node(UnExpr::uninit(match operator {
            UnaryOperationSymbol::Neg(_) => Neg,
            UnaryOperationSymbol::Not(_) => Not,
            UnaryOperationSymbol::Inv(_) => Inv,
            UnaryOperationSymbol::Plus(_) => Plus,
        }))
        .with_children_from([expr.as_ref()], context)
        .with_rlt(context, node)
        .id()
        .cast()
}

fn build_access(
    node: &rlt::Operation,
    builder: &mut SyntaxTreeBuilder,
    context: &mut (impl Linker + CodeHolder + Sized),
    left: &Box<rlt::Operation>,
    right: &Box<rlt::Operation>,
) -> NodeId<Operation> {
    builder
        .add_node(Acc::uninit())
        .with_children_from::<LEFT, _>([left.as_ref()], context)
        .with_children_from::<RIGHT, _>([right.as_ref()], context)
        .with_rlt(context, node)
        .id()
        .cast()
}

impl PopulateTree for rlt::Application {
    type Output = Appl;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(Appl::uninit())
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
                .add_node(Lambda::uninit())
                .with_children_from(binds.as_ref(), context)
                .with_children_from([expr.as_ref()], context)
                .with_rlt(context, self)
                .id()
                .cast(),
            rlt::Expression::Term(x) => x.convert(builder, context).cast(),
            rlt::Expression::Literal(x) => x.convert(builder, context).cast(),
            rlt::Expression::If(x) => x.convert(builder, context).cast::<CodeFlow>().cast(),
        }
    }
}
