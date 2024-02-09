use kodept::macro_context::DefaultContext;
use kodept::traversing::{Traversable, TraverseSet};
use kodept_interpret::SemanticAnalyzer;
use kodept_macros::erased::ErasedAnalyzer;
use kodept_macros::traits::{Context, UnrecoverableError};

pub struct PredefinedTraverseSet<C: Context = DefaultContext, E = UnrecoverableError>(
    TraverseSet<C, E>,
);

impl<C: Context> Default for PredefinedTraverseSet<C, UnrecoverableError> {
    fn default() -> Self {
        let mut set = TraverseSet::empty();
        set.add_independent(SemanticAnalyzer::new().erase());
        Self(set)
    }
}

impl<C: Context> Traversable<C, UnrecoverableError>
    for PredefinedTraverseSet<C, UnrecoverableError>
{
    fn traverse(&mut self, context: C) -> Result<C, (Vec<UnrecoverableError>, C)> {
        self.0.traverse(context)
    }
}
