use tracing::info;

use kodept_interpret::operator_desugaring::{
    AccessExpander, BinaryOperatorExpander, UnaryOperatorExpander,
};
use kodept_interpret::semantic_analyzer::ScopeAnalyzer;
use kodept_interpret::type_checker::TypeChecker;
use kodept_interpret::Witness;
use kodept_macros::traits::{MutableContext, UnrecoverableError};

use crate::steps::hlist::macros::{hlist, hlist_pat};
use crate::steps::pipeline::Pipeline;
use crate::steps::Step;

pub fn run_common_steps(ctx: &mut impl MutableContext) -> Result<(), UnrecoverableError> {
    info!("Step 1: Simplify AST");
    let hlist_pat![a, b, c] = Pipeline
        .step(hlist![
            AccessExpander,
            BinaryOperatorExpander,
            UnaryOperatorExpander
        ])
        .apply_with_context(ctx)?;

    info!("Step 2: Split by scopes and resolve symbols");
    let hlist_pat![scopes] = Pipeline
        .step(hlist![ScopeAnalyzer::new()])
        .apply_with_context(ctx)?;
    let scopes = scopes.into_inner();

    info!("Step 3: Infer and check types");
    Pipeline
        .step(hlist![TypeChecker::new(&scopes, Witness::fact(a, b, c))])
        .apply_with_context(ctx)?;

    Ok(())
}
