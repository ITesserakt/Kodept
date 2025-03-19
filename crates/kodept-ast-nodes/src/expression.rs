use crate::block_level::InitVar;
use crate::code_flow::IfExpr;
use crate::constants::Const;
use crate::literal::{Literal, Tuple};
use crate::properties::{Lhs, Rhs};
use crate::term::Ref;
use crate::types::{NonTyParam, TyParam};
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, relation, Str};
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

derive_node!(Exprs);
relation!(Exprs => children InitVar);
relation!(Exprs => children Const);
relation!(Exprs => children Exprs);
relation!(Exprs => children App);
relation!(Exprs => children Lambda);
relation!(Exprs => children IfExpr);
relation!(Exprs => children BinExpr);
relation!(Exprs => children UnExpr);
relation!(Exprs => children Ref);
relation!(Exprs => children Literal);
relation!(Exprs => children Tuple);

derive_node!(App);
relation!(App => or Lhs(optional Exprs));
relation!(App => or Lhs(optional App));
relation!(App => or Lhs(optional Lambda));
relation!(App => or Lhs(optional IfExpr));
relation!(App => or Lhs(optional BinExpr));
relation!(App => or Lhs(optional UnExpr));
relation!(App => or Lhs(optional Ref));
relation!(App => or Lhs(optional Literal));
relation!(App => or Lhs(optional Tuple));
relation!(App => or Rhs(children Exprs));
relation!(App => or Rhs(children App));
relation!(App => or Rhs(children Lambda));
relation!(App => or Rhs(children IfExpr));
relation!(App => or Rhs(children BinExpr));
relation!(App => or Rhs(children UnExpr));
relation!(App => or Rhs(children Ref));
relation!(App => or Rhs(children Literal));
relation!(App => or Rhs(children Tuple));

derive_node!(Lambda);
relation!(Lambda => children TyParam);
relation!(Lambda => children NonTyParam);
relation!(Lambda => child Exprs);

derive_node!(BinExpr);
relation!(BinExpr => or Lhs(optional Exprs));
relation!(BinExpr => or Lhs(optional App));
relation!(BinExpr => or Lhs(optional Lambda));
relation!(BinExpr => or Lhs(optional IfExpr));
relation!(BinExpr => or Lhs(optional BinExpr));
relation!(BinExpr => or Lhs(optional UnExpr));
relation!(BinExpr => or Lhs(optional Ref));
relation!(BinExpr => or Lhs(optional Literal));
relation!(BinExpr => or Lhs(optional Tuple));
relation!(BinExpr => or Rhs(children Exprs));
relation!(BinExpr => or Rhs(children App));
relation!(BinExpr => or Rhs(children Lambda));
relation!(BinExpr => or Rhs(children IfExpr));
relation!(BinExpr => or Rhs(children BinExpr));
relation!(BinExpr => or Rhs(children UnExpr));
relation!(BinExpr => or Rhs(children Ref));
relation!(BinExpr => or Rhs(children Literal));
relation!(BinExpr => or Rhs(children Tuple));

derive_node!(UnExpr);
relation!(UnExpr => optional Exprs);
relation!(UnExpr => optional App);
relation!(UnExpr => optional Lambda);
relation!(UnExpr => optional IfExpr);
relation!(UnExpr => optional BinExpr);
relation!(UnExpr => optional UnExpr);
relation!(UnExpr => optional Ref);
relation!(UnExpr => optional Literal);
relation!(UnExpr => optional Tuple);

impl FromSyntax for Exprs {
    type Syntax = ExpressionBlock;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, Exprs).with_children(source, pool, |scope| {
            scope.choose(Unit, node.expression.as_ref())
        })
    }
}

impl FromSyntax for App {
    type Syntax = Application;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, App).with_children(source, pool, |scope| {
            scope.choose::<_, _, Lhs>(Unit, [&node.expr]);
            scope.maybe_choose::<_, _, Rhs>(Unit, node.params.as_ref().map(|it| it.inner.as_ref()))
        })
    }
}

impl FromSyntax for Lambda {
    type Syntax = kodept_rlt::prelude::Lambda;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, Lambda).with_children(source, pool, |scope| {
            scope.with_builder(&*node.expr, |b| {
                ASTBuilder::from_queue(b, Exprs).with_children(|scope| {
                    scope.choose(Unit, [&*node.expr]);
                })
            });
            scope.choose(Unit, node.binds.inner.as_ref());
        })
    }
}

impl<R, Tag, A> Choose<Operation, R, Tag> for Unit
where
    Tag: Send + Sync + 'static,
    R: HasChild<Exprs, Tag, Arity = A>,
    R: HasChild<App, Tag, Arity = A>,
    R: HasChild<Lambda, Tag, Arity = A>,
    R: HasChild<IfExpr, Tag, Arity = A>,
    R: HasChild<BinExpr, Tag, Arity = A>,
    R: HasChild<UnExpr, Tag, Arity = A>,
    R: HasChild<Ref, Tag, Arity = A>,
    R: HasChild<Literal, Tag, Arity = A>,
    R: HasChild<Tuple, Tag, Arity = A>,
{
    type Arity = A;

    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Operation) -> ChildrenDisjoint<R, Source, A, Tag> {
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

fn build_unary_expression<R, Tag, A, S>(node: &Operation) -> ChildrenDisjoint<R, S, A, Tag>
where
    R: HasChild<UnExpr, Tag, Arity = A>,
    Tag: Send + Sync + 'static,
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

fn build_binary_expression<R, Tag, A, S>(node: &Operation) -> ChildrenDisjoint<R, S, A, Tag>
where
    R: HasChild<BinExpr, Tag, Arity = A>,
    Tag: Send + Sync + 'static,
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
            scope.choose::<_, _, Lhs>(Unit, [left.as_ref()]);
            scope.choose::<_, _, Rhs>(Unit, [right.as_ref()]);
        })
    })
}

fn build_access_expression<R, Tag, A, S>(node: &Operation) -> ChildrenDisjoint<R, S, A, Tag>
where
    R: HasChild<BinExpr, Tag, Arity = A>,
    Tag: Send + Sync + 'static,
    S: CodeHolder,
{
    ChildrenDisjoint::ad_hoc(node, |node, source, pool| {
        let Operation::Access { left, right, .. } = node else {
            unreachable!()
        };
        ASTBuilder::new(pool, BinExpr::Access).with_children(source, pool, |state| {
            state.choose::<_, _, Lhs>(Unit, [left.as_ref()]);
            state.choose::<_, _, Rhs>(Unit, [right.as_ref()])
        })
    })
}

impl<R, Tag, A> Choose<Expression, R, Tag> for Unit
where
    Tag: Send + Sync + 'static,
    R: HasChild<Lambda, Tag, Arity = A>,
    R: HasChild<IfExpr, Tag, Arity = A>,
    R: HasChild<Ref, Tag, Arity = A>,
    R: HasChild<Literal, Tag, Arity = A>,
    R: HasChild<Tuple, Tag, Arity = A>,
{
    type Arity = A;
    
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Expression) -> ChildrenDisjoint<R, Source, A, Tag> {
        match node {
            Expression::Lambda(x) => ChildrenDisjoint::new::<Lambda>(x),
            Expression::Term(x) => ChildrenDisjoint::new::<Ref>(x),
            Expression::Literal(x) => Unit::branch(x),
            Expression::If(x) => ChildrenDisjoint::new::<IfExpr>(x),
        }
    }
}
