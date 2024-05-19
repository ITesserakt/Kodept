use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use BinaryExpressionKind::*;
pub use expression_impl::*;
use kodept_core::structure::{rlt};
use kodept_core::structure::rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_core::structure::span::CodeHolder;
use UnaryExpressionKind::*;

use crate::{BlockLevel, node, UntypedParameter, wrapper};
use crate::graph::{GenericASTNode, NodeUnion};
use crate::graph::{Identity, SyntaxTreeBuilder};
use crate::graph::NodeId;
use crate::graph::tags::*;
use crate::macros::ForceInto;
use crate::traits::{Linker, PopulateTree};

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Operation {
        application(Application) = GenericASTNode::Application(x) => x.into(),
        access(Access) = GenericASTNode::Access(x) => x.into(),
        unary(Unary) = GenericASTNode::Unary(x) => x.into(),
        binary(Binary) = GenericASTNode::Binary(x) => x.into(),
        block(ExpressionBlock) = GenericASTNode::ExpressionBlock(x) => x.into(),
        expression(Expression) = n if Expression::contains(n) => n.force_into::<Expression>().into(),
    }
}

/// Manual macro expansion
mod expression_impl {
    use derive_more::{From, Into};
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    use crate::{IfExpression, Lambda, Literal, Term};
    use crate::graph::{GenericASTNode, NodeId, NodeUnion};
    use crate::macros::ForceInto;
    use crate::traits::Identifiable;
    use crate::utils::Skip;

    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    #[repr(transparent)]
    pub struct Expression(GenericASTNode);
    #[derive(derive_more::From)]
    pub enum ExpressionEnum<'lt> {
        Lambda(&'lt Lambda),
        If(&'lt IfExpression),
        Literal(&'lt Literal),
        Term(&'lt Term),
    }
    #[derive(derive_more::From)]
    pub enum ExpressionEnumMut<'lt> {
        Lambda(&'lt mut Lambda),
        If(&'lt mut IfExpression),
        Literal(&'lt mut Literal),
        Term(&'lt mut Term),
    }
    #[allow(unsafe_code)]
    unsafe impl NodeUnion for Expression {
        fn contains(node: &GenericASTNode) -> bool {
            #[allow(unused_variables)]
            #[allow(unreachable_patterns)]
            match node {
                GenericASTNode::Lambda(x) => true,
                GenericASTNode::If(x) => true,
                n if Literal::contains(n) => true,
                x if Term::contains(x) => true,
                _ => false,
            }
        }
    }
    impl<'a> TryFrom<&'a GenericASTNode> for &'a Expression {
        type Error = Skip<<&'a GenericASTNode as TryFrom<&'a GenericASTNode>>::Error>;

        #[inline]
        fn try_from(value: &'a GenericASTNode) -> Result<Self, Self::Error> {
            if !<Expression as NodeUnion>::contains(value) {
                return Err(Skip::Skipped);
            }
            Ok(<Expression as NodeUnion>::wrap(value))
        }
    }
    impl<'a> TryFrom<&'a mut GenericASTNode> for &'a mut Expression {
        type Error = Skip<<&'a mut GenericASTNode as TryFrom<&'a mut GenericASTNode>>::Error>;

        #[inline]
        fn try_from(value: &'a mut GenericASTNode) -> Result<Self, Self::Error> {
            if !<Expression as NodeUnion>::contains(value) {
                return Err(Skip::Skipped);
            }
            Ok(<Expression as NodeUnion>::wrap_mut(value))
        }
    }
    impl From<Lambda> for Expression {
        #[inline]
        fn from(value: Lambda) -> Self {
            let generic: GenericASTNode = value.into();
            Expression(generic)
        }
    }
    impl From<IfExpression> for Expression {
        #[inline]
        fn from(value: IfExpression) -> Self {
            let generic: GenericASTNode = value.into();
            Expression(generic)
        }
    }
    impl From<Literal> for Expression {
        #[inline]
        fn from(value: Literal) -> Self {
            let generic: GenericASTNode = value.into();
            Expression(generic)
        }
    }
    impl From<Term> for Expression {
        #[inline]
        fn from(value: Term) -> Self {
            let generic: GenericASTNode = value.into();
            Expression(generic)
        }
    }
    impl Identifiable for Expression {
        fn get_id(&self) -> NodeId<Self> {
            <GenericASTNode as Identifiable>::get_id(&self.0).narrow()
        }
    }
    impl Expression {
        pub fn as_enum(&self) -> ExpressionEnum {
            match self {
                Expression(GenericASTNode::Lambda(x)) => x.into(),
                Expression(GenericASTNode::If(x)) => x.into(),
                Expression(n) if Literal::contains(n) => n.force_into::<Literal>().into(),
                Expression(x) if Term::contains(x) => x.force_into::<Term>().into(),
                _ => unreachable!(),
            }
        }

        pub fn as_enum_mut(&mut self) -> ExpressionEnumMut {
            match self {
                Expression(GenericASTNode::Lambda(x)) => x.into(),
                Expression(GenericASTNode::If(x)) => x.into(),
                Expression(n) => {
                    if Literal::contains(n) {
                        n.force_into::<Literal>().into()
                    } else if Term::contains(n) {
                        n.force_into::<Term>().into()
                    } else {
                        unreachable!()
                    }
                }
            }
        }
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
    type Output = ExpressionBlock;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(ExpressionBlock::uninit())
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
        .add_node(Binary::uninit(match (operation, op_text) {
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
        .add_node(Unary::uninit(match operator {
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
        .add_node(Access::uninit())
        .with_children_from::<LEFT, _>([left.as_ref()], context)
        .with_children_from::<RIGHT, _>([right.as_ref()], context)
        .with_rlt(context, node)
        .id()
        .cast()
}

impl PopulateTree for rlt::Application {
    type Output = Application;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(Application::uninit())
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
            rlt::Expression::If(x) => x.convert(builder, context).cast(),
        }
    }
}
