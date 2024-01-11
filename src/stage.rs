use kodept::macro_context::DefaultContext;
use kodept::traversing::{Traversable, TraverseSet};
use kodept_interpret::SemanticAnalyzer;
use kodept_macros::erased::ErasedAnalyzer;
use kodept_macros::traits::{Context, UnrecoverableError};

pub struct PredefinedTraverseSet<'c, C: Context<'c> = DefaultContext<'c>, E = UnrecoverableError>(
    TraverseSet<'c, C, E>,
);

impl<'c, C: Context<'c>> Default for PredefinedTraverseSet<'c, C, UnrecoverableError> {
    fn default() -> Self {
        let mut set = TraverseSet::empty();
        set.add_independent(SemanticAnalyzer::new().erase());
        Self(set)
    }
}

impl<'c, C: Context<'c>> Traversable<'c, C, UnrecoverableError>
    for PredefinedTraverseSet<'c, C, UnrecoverableError>
{
    fn traverse(&mut self, context: C) -> Result<C, (Vec<UnrecoverableError>, C)> {
        self.0.traverse(context)
    }
}
