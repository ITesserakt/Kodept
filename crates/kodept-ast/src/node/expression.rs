#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::graph::tags::{LEFT, PRIMARY, RIGHT, SECONDARY};
use crate::graph::{Identity, SubSyntaxTree};
use crate::macros::implementation::node;
use crate::traits::PopulateTree;
use crate::{node_sub_enum, BlockLevel, CodeFlow, Lit, Param, Term};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_core::structure::span::CodeHolder;
use BinaryExpressionKind::*;
use UnaryExpressionKind::*;

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
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Appl {;
        pub expr: Identity<Operation> as PRIMARY,
        pub params: Vec<Operation> as SECONDARY,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Acc {;
        pub left: Identity<Operation> as LEFT,
        pub right: Identity<Operation> as RIGHT,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct UnExpr {
        pub kind: UnaryExpressionKind,;
        pub expr: Identity<Operation>,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct BinExpr {
        pub kind: BinaryExpressionKind,;
        pub left: Identity<Operation> as LEFT,
        pub right: Identity<Operation> as RIGHT,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Lambda {;
        // binds somehow wrapped in operation causing expr to fail => tags required
        pub binds: Vec<Param> as PRIMARY,
        pub expr: Identity<Operation> as SECONDARY,
    }
}

node! {
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
    Assign,
}

impl PopulateTree for rlt::ExpressionBlock {
    type Root = Exprs;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        SubSyntaxTree::new(Exprs::uninit().with_rlt(self))
            .with_children_from(self.expression.as_ref(), context)
    }
}

impl PopulateTree for rlt::Operation {
    type Root = Operation;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        match self {
            rlt::Operation::Block(x) => x.convert(context).cast(),
            rlt::Operation::Access { left, right, .. } => {
                build_access(self, context, left, right).cast()
            }
            rlt::Operation::TopUnary { operator, expr } => {
                build_unary(self, context, operator, expr).cast()
            }
            rlt::Operation::Binary {
                left,
                operation,
                right,
            } => build_binary(self, context, left, operation, right).cast(),
            rlt::Operation::Application(x) => x.convert(context).cast(),
            rlt::Operation::Expression(x) => x.convert(context).cast(),
        }
    }
}

fn build_binary<'a>(
    node: &'a rlt::Operation,
    context: &(impl CodeHolder + Sized),
    left: &'a rlt::Operation,
    operation: &'a BinaryOperationSymbol,
    right: &'a rlt::Operation,
) -> SubSyntaxTree<'a, BinExpr> {
    let binding = context.get_chunk_located(operation);
    let op_text = binding.as_ref();

    SubSyntaxTree::new(
        BinExpr::uninit(match (operation, op_text) {
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
            (BinaryOperationSymbol::Assign(_), "=") => Assign,

            (BinaryOperationSymbol::Mul(_), x) => panic!("Unknown mul operator found: {x}"),
            (BinaryOperationSymbol::Add(_), x) => panic!("Unknown add operator found: {x}"),
            (BinaryOperationSymbol::CompoundComparison(_), x) => {
                panic!("Unknown cmp operator found: {x}")
            }
            (BinaryOperationSymbol::Comparison(_), x) => panic!("Unknown cmp operator found: {x}"),
            (BinaryOperationSymbol::Bit(_), x) => panic!("Unknown bit operator found: {x}"),
            (BinaryOperationSymbol::Logic(_), x) => panic!("Unknown logic operator found: {x}"),
            (BinaryOperationSymbol::Assign(_), x) => panic!("Unknown assign operator found: {x}"),
        })
        .with_rlt(node),
    )
    .with_children_from::<LEFT, _>([left], context)
    .with_children_from::<RIGHT, _>([right], context)
}

fn build_unary<'a>(
    node: &'a rlt::Operation,
    context: &(impl CodeHolder + Sized),
    operator: &'a UnaryOperationSymbol,
    expr: &'a rlt::Operation,
) -> SubSyntaxTree<'a, UnExpr> {
    SubSyntaxTree::new(
        UnExpr::uninit(match operator {
            UnaryOperationSymbol::Neg(_) => Neg,
            UnaryOperationSymbol::Not(_) => Not,
            UnaryOperationSymbol::Inv(_) => Inv,
            UnaryOperationSymbol::Plus(_) => Plus,
        })
        .with_rlt(node),
    )
    .with_children_from([expr], context)
}

fn build_access<'a>(
    node: &'a rlt::Operation,
    context: &(impl CodeHolder + Sized),
    left: &'a rlt::Operation,
    right: &'a rlt::Operation,
) -> SubSyntaxTree<'a, Acc> {
    SubSyntaxTree::new(Acc::uninit().with_rlt(node))
        .with_children_from::<LEFT, _>([left], context)
        .with_children_from::<RIGHT, _>([right], context)
}

impl PopulateTree for rlt::Application {
    type Root = Appl;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        SubSyntaxTree::new(Appl::uninit().with_rlt(self))
            .with_children_from::<PRIMARY, _>([&self.expr], context)
            .with_children_from::<SECONDARY, _>(
                self.params
                    .as_ref()
                    .map_or([].as_slice(), |x| x.inner.as_ref()),
                context,
            )
    }
}

impl PopulateTree for rlt::Expression {
    type Root = Expression;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        match self {
            rlt::Expression::Lambda { binds, expr, .. } => {
                SubSyntaxTree::new(Lambda::uninit().with_rlt(self))
                    .with_children_from(binds.as_ref(), context)
                    .with_children_from([expr.as_ref()], context)
                    .cast()
            }
            rlt::Expression::Term(x) => x.convert(context).cast(),
            rlt::Expression::Literal(x) => x.convert(context).cast(),
            rlt::Expression::If(x) => x.convert(context).cast::<CodeFlow>().cast(),
        }
    }
}
