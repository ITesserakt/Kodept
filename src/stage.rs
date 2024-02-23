use std::io::stdout;

use kodept::macro_context::DefaultContext;
use kodept::traversing::TraverseSet;
use kodept_interpret::semantic_analyzer::SemanticAnalyzer;
use kodept_macros::default::ASTFormatter;
use kodept_macros::traits::{Context, UnrecoverableError};

pub struct PredefinedTraverseSet<C: Context = DefaultContext, E = UnrecoverableError>(
    TraverseSet<C, E>,
);

impl<C: Context> Default for PredefinedTraverseSet<C, UnrecoverableError> {
    fn default() -> Self {
        let mut set = TraverseSet::empty();
        set.add(SemanticAnalyzer::new()).then(|set, _| {
            set.add(ASTFormatter::new(stdout()));
        });
        Self(set)
    }
}

impl<C: Context, E> PredefinedTraverseSet<C, E> {
    pub fn into_inner(self) -> TraverseSet<C, E> {
        self.0
    }
}
