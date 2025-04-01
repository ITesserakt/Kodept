use kodept_ast::graph::tags;
use kodept_ast::traits::AsEnum;
use kodept_ast::utils::Skip;
use kodept_ast::utils::Skip::Skipped;
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{
    Acc, Appl, BinExpr, BinaryExpressionKind, Expression, Identifier, Operation, OperationEnumMut,
    Ref, ReferenceContext, Term, UnExpr, UnaryExpressionKind,
};
use std::convert::Infallible;
use BinaryExpressionKind::*;
use crate::macros::{Context, Macro, VisitGuard};

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
        let mut node = ctx
            .replace(id.cast::<Operation>(), Appl::uninit().map_into())
            .ok_or(Skipped)?;

        let name = node.use_value(|it| match it.as_enum() {
            OperationEnumMut::Binary(it) => match it.kind {
                Add => "__add_internal",
                Sub => "__sub_internal",
                Mul => "__mul_internal",
                Pow => "__pow_internal",
                Div => "__div_internal",
                Mod => "__mod_internal",
                Less => "__less_internal",
                LessEq => "__less_eq_internal",
                Greater => "__greater_internal",
                GreaterEq => "__greater_eq_internal",
                Eq => "__eq_internal",
                NEq => "__neq_internal",
                Or => "__or_internal",
                And => "__and_internal",
                Xor => "__xor_internal",
                Disj => "__disj_internal",
                Conj => "__conj_internal",
                ComplexComparison => "__cmp_internal",
                Assign => "__assign_internal",
            },
            _ => unreachable!(),
        });

        /*  BinExpr      Appl
        |     |   => |  |
        L     R      P  S
                        |\
                        L R */

        ctx.ast
            .update_children_tag::<_, _, Appl, _, { tags::LEFT }, { tags::SECONDARY }>(id);
        ctx.ast
            .update_children_tag::<_, _, Appl, _, { tags::RIGHT }, { tags::SECONDARY }>(id);
        let id = id.widen().coerce::<Appl>();
        let rlt = ctx.rlt.get_unknown(id).unwrap();
        ctx.add_child::<_, _, {tags::PRIMARY}>(
            id,
            Ref::uninit(
                ReferenceContext::global(["Prelude"]),
                Identifier::Reference { name: name.into(), },
            )
            .with_rlt(rlt)
            .map_into::<Term>()
            .map_into::<Expression>()
            .map_into::<Operation>(),
        );

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

        /*  UnExpr     Appl
        |     => |  |
        N        P  S */

        ctx.ast
            .update_children_tag::<_, _, Appl, _, { tags::NO_TAG }, { tags::SECONDARY }>(id);
        let id = id.widen().coerce::<Appl>();
        let rlt = ctx.rlt.get_unknown(id).unwrap();
        ctx.add_child::<_, _, { tags::PRIMARY }>(
            id,
            Ref::uninit(
                ReferenceContext::global(["Prelude"]),
                Identifier::Reference { name: name.into() },
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
        _guard: VisitGuard<Self::Node>,
        _ctx: &mut Self::Ctx<'_>,
    ) -> Result<(), Skip<Self::Error>> {
        Err(Skipped)
    }
}
