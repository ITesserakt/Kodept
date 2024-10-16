use kodept_ast::graph::tags;
use kodept_ast::interning::SharedStr;
use kodept_ast::traits::AsEnum;
use kodept_ast::utils::Skip;
use kodept_ast::utils::Skip::Skipped;
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{
    Acc, Appl, BinExpr, Expression, Identifier, Operation, OperationEnumMut, Ref, ReferenceContext,
    Term, UnExpr, UnaryExpressionKind,
};
use kodept_macros::context::Context;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::{Macro, MacroExt};
use std::convert::Infallible;

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
    ) -> Result<(), Skip<Self::Error>> {
        let id = guard.allow_only(VisitSide::Entering).ok_or(Skipped)?;
        let node = self.resolve(id, ctx);

        Ok(())
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
    ) -> Result<(), Skip<Self::Error>> {
        let id = guard.allow_only(VisitSide::Entering).ok_or(Skipped)?;

        let mut node = ctx
            .replace(id.cast::<Operation>(), Appl::uninit().map_into())
            .ok_or(Skipped)?;

        let name = node
            .use_value(|it| match it.as_enum() {
                OperationEnumMut::Unary(it) => match it.kind {
                    UnaryExpressionKind::Neg => "__neg_internal",
                    UnaryExpressionKind::Not => "__not_internal",
                    UnaryExpressionKind::Inv => "__inv_internal",
                    UnaryExpressionKind::Plus => "__plus_internal",
                },
                _ => unreachable!(),
            })
            .to_string();

        ctx.ast
            .update_children_tag::<_, _, Appl, _, { tags::NO_TAG }, { tags::SECONDARY }>(id);
        let id = id.widen().coerce::<Appl>();
        let rlt = ctx.rlt.get_unknown(id).unwrap();
        ctx.add_child::<_, _, { tags::PRIMARY }>(
            id,
            Ref::uninit(
                ReferenceContext::global(["Prelude"]),
                Identifier::Reference {
                    name: SharedStr::new(name),
                },
            )
            .with_rlt(rlt)
            .map_into::<Term>()
            .map_into::<Expression>()
            .map_into::<Operation>(),
        );

        Ok(())
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
    ) -> Result<(), Skip<Self::Error>> {
        let id = guard.allow_only(VisitSide::Entering).ok_or(Skipped)?;
        let node = self.resolve(id, ctx);

        Ok(())
    }
}
