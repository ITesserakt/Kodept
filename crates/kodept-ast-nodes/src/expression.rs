use crate::block_level::InitVar;
use crate::code_flow::IfExpr;
use crate::function::Func;
use crate::literal::{Literal, Tuple};
use crate::properties::{BlockLevel, Expr, LeftExpr, Param, RightExpr};
use crate::term::Ref;
use crate::types::{NonTyParam, TyParam};
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::properties::tags::Tagged;
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, Str};
use kodept_rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_rlt::prelude::{Application, Expression, ExpressionBlock, Operation};
use std::ops::Deref;

#[derive(Debug, PartialEq, Component)]
pub struct Exprs;

#[derive(Debug, PartialEq, Component)]
pub struct App;

#[derive(Debug, PartialEq, Component)]
pub struct Lambda;

#[derive(Debug, PartialEq, Component)]
pub enum BinExpr {
    Access,
    Add,
    Sub,
    Mul,
    Pow,
    Div,
    Mod,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Eq,
    NEq,
    Or,
    And,
    Xor,
    Disj,
    Conj,
    ComplexComparison,
    Assign,
}

#[derive(Debug, PartialEq, Component)]
pub enum UnExpr {
    Neg,
    Not,
    Inv,
    Plus,
}

derive_node!(Exprs {
    relations = [
        children InitVar where tag = BlockLevel,
        children Func where tag = BlockLevel,
        children Exprs where tag = BlockLevel,
        children App where tag = BlockLevel,
        children Lambda where tag = BlockLevel,
        children IfExpr where tag = BlockLevel,
        children BinExpr where tag = BlockLevel,
        children UnExpr where tag = BlockLevel,
        children Ref where tag = BlockLevel,
        children Literal where tag = BlockLevel,
        children Tuple where tag = BlockLevel,
    ],
    properties = []
});
derive_node!(App {
    relations = [
        optional Exprs where tag = LeftExpr,
        optional App where tag = LeftExpr,
        optional Lambda where tag = LeftExpr,
        optional IfExpr where tag = LeftExpr,
        optional BinExpr where tag = LeftExpr,
        optional UnExpr where tag = LeftExpr,
        optional Ref where tag = LeftExpr,
        optional Literal where tag = LeftExpr,
        optional Tuple where tag = LeftExpr,

        children Exprs where tag = RightExpr,
        children App where tag = RightExpr,
        children Lambda where tag = RightExpr,
        children IfExpr where tag = RightExpr,
        children BinExpr where tag = RightExpr,
        children UnExpr where tag = RightExpr,
        children Ref where tag = RightExpr,
        children Literal where tag = RightExpr,
        children Tuple where tag = RightExpr,
    ],
    properties = []
});
derive_node!(Lambda {
    relations = [
        children TyParam where tag = Param,
        children NonTyParam where tag = Param,

        optional Exprs where tag = Expr,
        optional App where tag = Expr,
        optional Lambda where tag = Expr,
        optional IfExpr where tag = Expr,
        optional BinExpr where tag = Expr,
        optional UnExpr where tag = Expr,
        optional Ref where tag = Expr,
        optional Literal where tag = Expr,
        optional Tuple where tag = Expr,
    ],
    properties = []
});
derive_node!(BinExpr {
    relations = [
        children Exprs where tag = LeftExpr,
        children App where tag = LeftExpr,
        children Lambda where tag = LeftExpr,
        children IfExpr where tag = LeftExpr,
        children BinExpr where tag = LeftExpr,
        children UnExpr where tag = LeftExpr,
        children Ref where tag = LeftExpr,
        children Literal where tag = LeftExpr,
        children Tuple where tag = LeftExpr,

        children Exprs where tag = RightExpr,
        children App where tag = RightExpr,
        children Lambda where tag = RightExpr,
        children IfExpr where tag = RightExpr,
        children BinExpr where tag = RightExpr,
        children UnExpr where tag = RightExpr,
        children Ref where tag = RightExpr,
        children Literal where tag = RightExpr,
        children Tuple where tag = RightExpr,
    ],
    properties = []
});
derive_node!(UnExpr {
    relations = [
        children Exprs where tag = Expr,
        children App where tag = Expr,
        children Lambda where tag = Expr,
        children IfExpr where tag = Expr,
        children BinExpr where tag = Expr,
        children UnExpr where tag = Expr,
        children Ref where tag = Expr,
        children Literal where tag = Expr,
        children Tuple where tag = Expr,
    ],
    properties = []
});

impl FromSyntax for Exprs {
    type Syntax = ExpressionBlock;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, Exprs).with_children(source, pool, |scope| {
            scope.choose(Unit, node.expression.as_ref())
        })
    }
}

impl FromSyntax for App {
    type Syntax = Application;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, App).with_children(source, pool, |scope| {
            scope.choose::<_, _, LeftExpr>(Unit, [&node.expr]);
            scope.maybe_choose::<_, _, RightExpr>(
                Unit,
                node.params.as_ref().map(|it| it.inner.as_ref()),
            )
        })
    }
}

impl FromSyntax for Lambda {
    type Syntax = kodept_rlt::prelude::Lambda;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, Lambda).with_children(source, pool, |scope| {
            scope.choose(Unit, [&*node.expr]);
            scope.choose(Unit, node.binds.inner.as_ref());
        })
    }
}

impl<R, Tag> Choose<Operation, R, Tag> for Unit
where
    Tag: Tagged,
    R: HasChild<Exprs, Tag>,
    R: HasChild<App, Tag>,
    R: HasChild<Lambda, Tag>,
    R: HasChild<IfExpr, Tag>,
    R: HasChild<BinExpr, Tag>,
    R: HasChild<UnExpr, Tag>,
    R: HasChild<Ref, Tag>,
    R: HasChild<Literal, Tag>,
    R: HasChild<Tuple, Tag>,
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Operation) -> ChildrenDisjoint<R, Source, Tag> {
        match node {
            Operation::Block(x) => ChildrenDisjoint::new::<Exprs>(x),
            Operation::Access { .. } => build_access_expression(node),
            Operation::Unary { .. } => build_unary_expression(node),
            Operation::Binary { .. } => build_binary_expression(node),
            Operation::Application(x) => ChildrenDisjoint::new::<App>(x),
            Operation::Expression(x) => Unit::branch(x),
        }
    }
}

fn build_unary_expression<R, Tag, S>(node: &Operation) -> ChildrenDisjoint<R, S, Tag>
where
    R: HasChild<UnExpr, Tag>,
    Tag: Tagged,
    S: CodeHolder,
{
    ChildrenDisjoint::ad_hoc(node, |node, source, pool| {
        let Operation::Unary { operator, expr } = node else {
            unreachable!()
        };
        let value = match operator {
            UnaryOperationSymbol::Neg(_) => UnExpr::Neg,
            UnaryOperationSymbol::Not(_) => UnExpr::Not,
            UnaryOperationSymbol::Inv(_) => UnExpr::Inv,
            UnaryOperationSymbol::Plus(_) => UnExpr::Plus,
        };
        ASTBuilder::new(pool, value).with_children(source, pool, |scope| {
            scope.choose(Unit, [expr.as_ref()]);
        })
    })
}

fn build_binary_expression<R, Tag, S>(node: &Operation) -> ChildrenDisjoint<R, S, Tag>
where
    R: HasChild<BinExpr, Tag>,
    Tag: Tagged,
    S: CodeHolder,
{
    ChildrenDisjoint::ad_hoc(node, |node, source: S, pool| {
        let Operation::Binary {
            left,
            operation,
            right,
        } = node
        else {
            unreachable!()
        };
        let op_text: Str = source.get_chunk_located(operation);
        let value = match (operation, op_text.deref()) {
            (BinaryOperationSymbol::Pow(_), _) => BinExpr::Pow,
            (BinaryOperationSymbol::Mul(_), "*") => BinExpr::Mul,
            (BinaryOperationSymbol::Mul(_), "/") => BinExpr::Div,
            (BinaryOperationSymbol::Mul(_), "%") => BinExpr::Mod,
            (BinaryOperationSymbol::Add(_), "+") => BinExpr::Add,
            (BinaryOperationSymbol::Add(_), "-") => BinExpr::Add,
            (BinaryOperationSymbol::ComplexComparison(_), _) => BinExpr::ComplexComparison,
            (BinaryOperationSymbol::CompoundComparison(_), "<=") => BinExpr::LessEq,
            (BinaryOperationSymbol::CompoundComparison(_), ">=") => BinExpr::GreaterEq,
            (BinaryOperationSymbol::CompoundComparison(_), "!=") => BinExpr::NEq,
            (BinaryOperationSymbol::CompoundComparison(_), "==") => BinExpr::Eq,
            (BinaryOperationSymbol::Comparison(_), "<") => BinExpr::Less,
            (BinaryOperationSymbol::Comparison(_), ">") => BinExpr::Greater,
            (BinaryOperationSymbol::Bit(_), "|") => BinExpr::Or,
            (BinaryOperationSymbol::Bit(_), "&") => BinExpr::And,
            (BinaryOperationSymbol::Bit(_), "^") => BinExpr::Xor,
            (BinaryOperationSymbol::Logic(_), "||") => BinExpr::Disj,
            (BinaryOperationSymbol::Logic(_), "&&") => BinExpr::Conj,
            (BinaryOperationSymbol::Assign(_), _) => BinExpr::Assign,
            _ => unreachable!(),
        };
        ASTBuilder::new(pool, value).with_children(source, pool, |scope| {
            scope.choose::<_, _, LeftExpr>(Unit, [left.as_ref()]);
            scope.choose::<_, _, RightExpr>(Unit, [right.as_ref()]);
        })
    })
}

fn build_access_expression<R, Tag, S>(node: &Operation) -> ChildrenDisjoint<R, S, Tag>
where
    R: HasChild<BinExpr, Tag>,
    Tag: Tagged,
    S: CodeHolder,
{
    ChildrenDisjoint::ad_hoc(node, |node, source, pool| {
        let Operation::Access { left, right, .. } = node else {
            unreachable!()
        };
        ASTBuilder::new(pool, BinExpr::Access).with_children(source, pool, |state| {
            state.choose::<_, _, LeftExpr>(Unit, [left.as_ref()]);
            state.choose::<_, _, RightExpr>(Unit, [right.as_ref()])
        })
    })
}

impl<R, Tag> Choose<Expression, R, Tag> for Unit
where
    Tag: Tagged,
    R: HasChild<Lambda, Tag>,
    R: HasChild<IfExpr, Tag>,
    R: HasChild<Ref, Tag>,
    R: HasChild<Literal, Tag>,
    R: HasChild<Tuple, Tag>,
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Expression) -> ChildrenDisjoint<R, Source, Tag> {
        match node {
            Expression::Lambda(x) => ChildrenDisjoint::new::<Lambda>(x),
            Expression::Term(x) => ChildrenDisjoint::new::<Ref>(x),
            Expression::Literal(x) => Unit::branch(x),
            Expression::If(x) => ChildrenDisjoint::new::<IfExpr>(x),
        }
    }
}
