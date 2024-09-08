use std::convert::Infallible;

use kodept_ast::graph::{tags, AnyNode, Change, ChangeSet};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{
    Acc, Appl, BinExpr, BinaryExpressionKind, BitKind, ComparisonKind, EqKind, Expression,
    Identifier, LogicKind, MathKind, Operation, Ref, ReferenceContext, Term, UnExpr,
    UnaryExpressionKind,
};
use kodept_macros::context::{Context, SyntaxProvider};
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::Macro;

#[derive(Default)]
pub struct BinaryOperatorExpander;

#[derive(Default)]
pub struct UnaryOperatorExpander;

#[derive(Default)]
pub struct AccessExpander;

fn replace_with<N: Identifiable + Into<AnyNode>>(
    replaced: &N,
    function_name: &'static str,
) -> ChangeSet {
    // ::Prelude::<function_name>(<left>, <right>)
    let id = replaced.get_id().widen();

    ChangeSet::from_iter([
        Change::replace(id, Appl::uninit()),
        Change::add::<_, _, { tags::PRIMARY }>(
            id.narrow::<Appl>(),
            Ref::uninit(
                ReferenceContext::global(["Prelude"]),
                Identifier::Reference {
                    name: function_name.to_string(),
                },
            )
            .map_into::<Term>()
            .map_into::<Expression>()
            .map_into::<Operation>(),
        ),
    ])
}

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

impl<C> Macro<C> for BinaryOperatorExpander
where
    C: SyntaxProvider,
{
    type Error = Infallible;
    type Node = BinExpr;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut impl Context<C>,
    ) -> Execution<Self::Error, ChangeSet> {
        let id = guard.allow_only(VisitSide::Entering)?;
        let node = self.resolve(id, ctx);

        Execution::Completed(match &node.kind {
            BinaryExpressionKind::Math(x) => match x {
                MathKind::Add => replace_with(node, "__add_internal"),
                MathKind::Sub => replace_with(node, "__sub_internal"),
                MathKind::Mul => replace_with(node, "__mul_internal"),
                MathKind::Pow => replace_with(node, "__pow_internal"),
                MathKind::Div => replace_with(node, "__div_internal"),
                MathKind::Mod => replace_with(node, "__mod_internal"),
            },
            BinaryExpressionKind::Cmp(x) => match x {
                ComparisonKind::Less => replace_with(node, "__less_internal"),
                ComparisonKind::LessEq => replace_with(node, "__less_eq_internal"),
                ComparisonKind::Greater => replace_with(node, "__greater_internal"),
                ComparisonKind::GreaterEq => replace_with(node, "__greater_internal"),
            },
            BinaryExpressionKind::Eq(x) => match x {
                EqKind::Eq => replace_with(node, "__eq_internal"),
                EqKind::NEq => replace_with(node, "__neq_internal"),
            },
            BinaryExpressionKind::Bit(x) => match x {
                BitKind::Or => replace_with(node, "__or_internal"),
                BitKind::And => replace_with(node, "__and_internal"),
                BitKind::Xor => replace_with(node, "__xor_internal"),
            },
            BinaryExpressionKind::Logic(x) => match x {
                LogicKind::Disj => replace_with(node, "__dis_internal"),
                LogicKind::Conj => replace_with(node, "__con_internal"),
            },
            BinaryExpressionKind::ComplexComparison => replace_with(node, "__cmp_internal"),
            BinaryExpressionKind::Assign => replace_with(node, "__assign_internal"),
        })
    }
}

impl<C: SyntaxProvider> Macro<C> for UnaryOperatorExpander {
    type Error = Infallible;
    type Node = UnExpr;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut impl Context<C>,
    ) -> Execution<Self::Error, ChangeSet> {
        let id = guard.allow_only(VisitSide::Entering)?;
        let node = self.resolve(id, ctx);

        Execution::Completed(match node.kind {
            UnaryExpressionKind::Neg => replace_with(node, "__neg_internal"),
            UnaryExpressionKind::Not => replace_with(node, "__not_internal"),
            UnaryExpressionKind::Inv => replace_with(node, "__inv_internal"),
            UnaryExpressionKind::Plus => replace_with(node, "__plus_internal"),
        })
    }
}

impl<C: SyntaxProvider> Macro<C> for AccessExpander {
    type Error = Infallible;
    type Node = Acc;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut impl Context<C>,
    ) -> Execution<Self::Error, ChangeSet> {
        let id = guard.allow_only(VisitSide::Entering)?;
        let node = self.resolve(id, ctx);

        Execution::Completed(replace_with(node, "compose"))
    }
}
