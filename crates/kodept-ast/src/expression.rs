use crate::node_id::NodeId;
use crate::traits::{
    IdProducer, Identifiable, Instantiable, IntoAst, LinkGuard, Linker, NewInstance,
};
use crate::{impl_identifiable, BlockLevel, Identifier, IfExpression, Literal, Term};
use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Application {
    pub expr: Operation,
    pub params: Vec<Operation>,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Access {
    pub left: Operation,
    pub right: Operation,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Unary {
    pub kind: UnaryExpressionKind,
    pub expr: Operation,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Binary {
    pub left: Operation,
    pub right: Operation,
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
    pub binds: Vec<Identifier>,
    pub expr: Operation,
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
    pub items: Vec<BlockLevel>,
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

impl_identifiable! {
    Application,
    Access,
    Unary,
    Binary,
    Lambda,
    ExpressionBlock
}
node_group!(family: ExpressionBlock, nodes: [ExpressionBlock, BlockLevel]);
node_group!(family: Operation, nodes: [
    Operation, Application, Access, Unary, Binary, Expression, ExpressionBlock
]);
node_group!(family: Expression, nodes: [
    Expression, Lambda, Term, Literal, IfExpression
]);
node_group!(family: Lambda, nodes: [Lambda, Identifier]);

impl NewInstance for ExpressionBlock {
    type Constructor = Vec<BlockLevel>;

    fn instantiate<P: IdProducer>(init: Self::Constructor, ctx: &mut P) -> LinkGuard<Self> {
        Self {
            items: init,
            id: ctx.next_id(),
        }
        .guard()
    }
}

impl Identifiable for Expression {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            Expression::Lambda(x) => x.get_id().cast(),
            Expression::Term(x) => x.get_id().cast(),
            Expression::Literal(x) => x.get_id().cast(),
            Expression::If(x) => x.get_id().cast(),
        }
    }
}

impl Identifiable for Operation {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            Operation::Application(x) => x.get_id().cast(),
            Operation::Access(x) => x.get_id().cast(),
            Operation::Unary(x) => x.get_id().cast(),
            Operation::Binary(x) => x.get_id().cast(),
            Operation::Expression(x) => x.get_id().cast(),
            Operation::Block(x) => x.get_id().cast(),
        }
    }
}

impl IntoAst for rlt::ExpressionBlock {
    type Output = ExpressionBlock;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = ExpressionBlock {
            items: self
                .expression
                .iter()
                .map(|it| it.construct(context))
                .collect(),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

fn new_unary<'x, P: IdProducer + Linker<'x> + CodeHolder>(
    operator: &UnaryOperationSymbol,
    expr: &'x rlt::Operation,
    context: &mut P,
) -> Unary {
    let kind = match operator {
        UnaryOperationSymbol::Neg(_) => UnaryExpressionKind::Neg,
        UnaryOperationSymbol::Not(_) => UnaryExpressionKind::Not,
        UnaryOperationSymbol::Inv(_) => UnaryExpressionKind::Inv,
        UnaryOperationSymbol::Plus(_) => UnaryExpressionKind::Plus,
    };
    Unary {
        kind,
        expr: expr.construct(context),
        id: context.next_id(),
    }
}

fn new_binary<'x, P: IdProducer + Linker<'x> + CodeHolder>(
    left: &'x rlt::Operation,
    right: &'x rlt::Operation,
    operator: &BinaryOperationSymbol,
    context: &mut P,
) -> Binary {
    let kind = match operator {
        BinaryOperationSymbol::Pow(_) => BinaryExpressionKind::Pow,
        BinaryOperationSymbol::Mul(_) => BinaryExpressionKind::Mul,
        BinaryOperationSymbol::Add(_) => BinaryExpressionKind::Add,
        BinaryOperationSymbol::ComplexComparison(_) => BinaryExpressionKind::ComplexComparison,
        BinaryOperationSymbol::CompoundComparison(_) => BinaryExpressionKind::CompoundComparison,
        BinaryOperationSymbol::Comparison(_) => BinaryExpressionKind::Comparison,
        BinaryOperationSymbol::Bit(_) => BinaryExpressionKind::Bit,
        BinaryOperationSymbol::Logic(_) => BinaryExpressionKind::Logic,
    };
    Binary {
        left: left.construct(context),
        right: right.construct(context),
        kind,
        id: context.next_id(),
    }
}

impl IntoAst for rlt::Operation {
    type Output = Operation;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = match self {
            rlt::Operation::Block(x) => Box::new(x.construct(context)).into(),
            rlt::Operation::Access { left, right, .. } => {
                let node = Access {
                    left: left.construct(context),
                    right: right.construct(context),
                    id: context.next_id(),
                };
                Box::new(context.link(node, self)).into()
            }
            rlt::Operation::TopUnary { operator, expr } => {
                let node = new_unary(operator, expr, context);
                Box::new(context.link(node, self)).into()
            }
            rlt::Operation::Binary {
                left,
                operation,
                right,
            } => {
                let node = new_binary(left, right, operation, context);
                Box::new(context.link(node, self)).into()
            }
            rlt::Operation::Application(x) => Box::new(x.construct(context)).into(),
            rlt::Operation::Expression(x) => Box::new(x.construct(context)).into(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::Application {
    type Output = Application;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = Application {
            expr: self.expr.construct(context),
            params: self.params.as_ref().map_or(vec![], |it| {
                it.inner.iter().map(|it| it.construct(context)).collect()
            }),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::Expression {
    type Output = Expression;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = match self {
            rlt::Expression::Lambda { binds, expr, .. } => {
                let node = Lambda {
                    id: context.next_id(),
                    expr: expr.construct(context),
                    binds: binds.iter().map(|it| it.construct(context)).collect(),
                };
                context.link(node, self).into()
            }
            rlt::Expression::Term(x) => x.construct(context).into(),
            rlt::Expression::Literal(x) => x.construct(context).into(),
            rlt::Expression::If(x) => x.construct(context).into(),
        };
        context.link(node, self)
    }
}

impl Instantiable for Application {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            expr: self.expr.new_instance(context),
            params: self
                .params
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Access {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            left: self.left.new_instance(context),
            right: self.right.new_instance(context),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Unary {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            kind: self.kind.clone(),
            expr: self.expr.new_instance(context),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Binary {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            left: self.left.new_instance(context),
            right: self.right.new_instance(context),
            kind: self.kind.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Lambda {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            id: context.next_id(),
            binds: self
                .binds
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            expr: self.expr.new_instance(context),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for ExpressionBlock {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            items: self
                .items
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Expression {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Expression::Lambda(x) => x.new_instance(context).into(),
            Expression::Term(x) => x.new_instance(context).into(),
            Expression::Literal(x) => x.new_instance(context).into(),
            Expression::If(x) => x.new_instance(context).into(),
        }
    }
}

impl Instantiable for Operation {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Operation::Application(x) => Box::new(x.new_instance(context)).into(),
            Operation::Access(x) => Box::new(x.new_instance(context)).into(),
            Operation::Unary(x) => Box::new(x.new_instance(context)).into(),
            Operation::Binary(x) => Box::new(x.new_instance(context)).into(),
            Operation::Expression(x) => Box::new(x.new_instance(context)).into(),
            Operation::Block(x) => Box::new(x.new_instance(context)).into(),
        }
    }
}
