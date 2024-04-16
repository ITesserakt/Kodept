use std::rc::Rc;

use kodept::macro_context::DefaultContext;
use kodept::traversing::TraverseSet;
use kodept_interpret::operator_desugaring::BinaryOperatorExpander;
use kodept_interpret::semantic_analyzer::ScopeAnalyzer;
use kodept_interpret::type_checker::TypeChecker;
use kodept_macros::traits::{Context, UnrecoverableError};

pub struct PredefinedTraverseSet<C: Context = DefaultContext, E = UnrecoverableError>(
    TraverseSet<C, E>,
);

impl<C: Context + 'static> Default for PredefinedTraverseSet<C, UnrecoverableError> {
    fn default() -> Self {
        let mut set = TraverseSet::empty();

        set.dependency(BinaryOperatorExpander::new())
            .then(|set, _| {
                set.dependency(ScopeAnalyzer::new())
                    .then(|set, sa| {
                        let scopes = Rc::new(sa.into_inner());
                        set.dependency(TypeChecker::new(scopes.clone())).add(set);
                    })
                    .add(set);
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
