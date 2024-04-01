use kodept_ast::{Application, Binary, BinaryExpressionKind, Reference};
use kodept_ast::graph::{Change, ChangeSet, GenericASTNode, tags};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_macros::Macro;
use kodept_macros::traits::{Context, UnrecoverableError};

#[derive(Default)]
pub struct BinaryOperatorExpander {}

fn replace_with<N: Identifiable>(replaced: &N, function_name: &'static str) -> ChangeSet
where
    GenericASTNode: TryFrom<N>,
{
    // <function_name>(<left>, <right>)
    let id = replaced.get_id().cast();

    ChangeSet::from_iter([
        Change::replace(id, Application::new()),
        Change::add(
            id,
            Reference::new_ref(function_name.to_string()),
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
    type Error = UnrecoverableError;
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
