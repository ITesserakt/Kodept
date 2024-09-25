use std::convert::Infallible;

use kodept_ast::graph::{tags, AnyNode};
use kodept_ast::traits::Identifiable;
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{
    Acc, Appl, BinExpr, BinaryExpressionKind, BitKind, ComparisonKind, EqKind, Expression,
    Identifier, LogicKind, MathKind, Operation, Ref, ReferenceContext, Term, UnExpr,
    UnaryExpressionKind,
};
use kodept_macros::context::Context;
use kodept_macros::execution::Execution;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::{Macro, MacroExt};
use kodept_macros::execution::Execution::Completed;

#[derive(Default)]
pub struct BinaryOperatorExpander;

#[derive(Default)]
pub struct UnaryOperatorExpander;

#[derive(Default)]
pub struct AccessExpander;

impl BinaryOperatorExpander {
    pub fn new() -> Self {
        Self
    }
}

impl UnaryOperatorExpander {
    pub fn new() -> Self {
        Self
    }
}

impl AccessExpander {
    pub fn new() -> Self {
        Self
    }
}

impl Macro for BinaryOperatorExpander {
    type Error = Infallible;
    type Node = BinExpr;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Execution<Self::Error> {
        let id = guard.allow_only(VisitSide::Entering)?;
        let node = self.resolve(id, ctx);
        
        Completed(())
    }
}

impl Macro for UnaryOperatorExpander {
    type Error = Infallible;
    type Node = UnExpr;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Execution<Self::Error> {
        let id = guard.allow_only(VisitSide::Entering)?;
        let node = self.resolve(id, ctx);
        
        Completed(())
    }
}

impl Macro for AccessExpander {
    type Error = Infallible;
    type Node = Acc;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Execution<Self::Error> {
        let id = guard.allow_only(VisitSide::Entering)?;
        let node = self.resolve(id, ctx);

        Completed(())
    }
}
