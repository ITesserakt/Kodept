use crate::steps::pipeline::Pipeline;
use crate::steps::Step;
use derive_more::Constructor;
use kodept_interpret::operator_desugaring::{
    AccessExpander, BinaryOperatorExpander, UnaryOperatorExpander,
};
use kodept_interpret::semantic_analyzer::ScopeAnalyzer;
use kodept_macros::context::Context;
use std::num::NonZeroU16;
use tracing::info;
use kodept_interpret::type_checker::TypeChecker;

#[derive(Constructor)]
pub struct Config {
    pub recursion_depth: NonZeroU16,
}

pub fn run_common_steps(
    ctx: &mut Context,
    config: &Config,
) -> Option<()> {
    info!("Step 1: Simplify AST");
    let (_, _, _) = Pipeline
        .define_step((
            AccessExpander::new(),
            BinaryOperatorExpander::new(),
            UnaryOperatorExpander::new(),
        ))
        .apply_with_context(ctx)?;

    info!("Step 2: Split by scopes and resolve symbols");
    let (scopes,) = Pipeline
        .define_step((ScopeAnalyzer::new(),))
        .apply_with_context(ctx)?;
    let scopes = scopes.into_inner();
    
    info!("Step 3: Infer and check types");
    let (_,) = Pipeline
        .define_step((TypeChecker::new(&scopes, config.recursion_depth),))
        .apply_with_context(ctx)?;

    Some(())
}
