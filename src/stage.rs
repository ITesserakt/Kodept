use kodept::macro_context::DefaultContext;
use kodept::traversing::TraverseSet;
use kodept_interpret::semantic_analyzer::SemanticAnalyzer;
use kodept_interpret::type_checker::TypeChecker;
use kodept_macros::traits::{Context, UnrecoverableError};

pub struct PredefinedTraverseSet<C: Context = DefaultContext, E = UnrecoverableError>(
    TraverseSet<C, E>,
);

impl<C: Context + 'static> Default for PredefinedTraverseSet<C, UnrecoverableError> {
    fn default() -> Self {
        let mut set = TraverseSet::empty();
        set.dependency(SemanticAnalyzer::new())
            .then(|set, sa| {
                set.dependency(TypeChecker::new(sa.into_inner())).add(set);
            })
            .add(&mut set);

        Self(set)
    }
}

impl<C: Context, E> PredefinedTraverseSet<C, E> {
    pub fn into_inner(self) -> TraverseSet<C, E> {
        self.0
    }
}
