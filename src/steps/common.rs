use std::num::NonZeroU16;
use derive_more::Constructor;
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

#[derive(Constructor)]
pub struct Config {
    pub recursion_depth: NonZeroU16,
}

pub fn run_common_steps(
    ctx: &mut impl MutableContext,
    config: &Config,
) -> Result<(), UnrecoverableError> {
    info!("Step 1: Simplify AST");
    let hlist_pat![a, b, c] = Pipeline
        .define_step(hlist![
            AccessExpander::new(),
            BinaryOperatorExpander::new(),
            UnaryOperatorExpander::new()
        ])
        .apply_with_context(ctx)?;

    info!("Step 2: Split by scopes and resolve symbols");
    let hlist_pat![scopes] = Pipeline
        .define_step(hlist![ScopeAnalyzer::new()])
        .apply_with_context(ctx)?;
    let scopes = scopes.into_inner();

    info!("Step 3: Infer and check types");
    Pipeline
        .define_step(hlist![TypeChecker::new(
            &scopes,
            config.recursion_depth,
            Witness::fact(a, b, c)
        )])
        .apply_with_context(ctx)?;

    Ok(())
}
