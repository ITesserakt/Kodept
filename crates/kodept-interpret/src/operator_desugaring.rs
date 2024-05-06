use std::convert::Infallible;

use kodept_ast::graph::{tags, Change, ChangeSet, GenericASTNode};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_ast::{Application, Binary, BinaryExpressionKind, Identifier, Reference};
use kodept_macros::traits::Context;
use kodept_macros::Macro;

#[derive(Default)]
pub struct BinaryOperatorExpander {}

fn replace_with<N: Identifiable + Into<GenericASTNode>>(
    replaced: &N,
    function_name: &'static str,
) -> ChangeSet {
    // <function_name>(<left>, <right>)
    let id = replaced.get_id().widen();

    ChangeSet::from_iter([
        Change::replace(id, Application::uninit()),
        Change::add(
            id,
            Reference::uninit(Identifier::Reference {
                name: function_name.to_string(),
            }),
            tags::PRIMARY,
        ),
    ])
}

impl BinaryOperatorExpander {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Macro for BinaryOperatorExpander {
    type Error = Infallible;
    type Node = Binary;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        _context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        if !matches!(side, VisitSide::Exiting | VisitSide::Leaf) {
            Execution::Skipped?;
        }

        Execution::Completed(match node.kind {
            BinaryExpressionKind::Pow => replace_with(&*node, "pow"),
            BinaryExpressionKind::Mul => replace_with(&*node, "mul"),
            BinaryExpressionKind::Add => replace_with(&*node, "add"),
            BinaryExpressionKind::ComplexComparison => replace_with(&*node, "cmp"),
            BinaryExpressionKind::CompoundComparison => replace_with(&*node, "cmp"),
            BinaryExpressionKind::Comparison => replace_with(&*node, "cmp"),
            BinaryExpressionKind::Bit => replace_with(&*node, "bit"),
            BinaryExpressionKind::Logic => replace_with(&*node, "lgc"),
        })
    }
}
