use crate::steps::pipeline::Pipeline;
use crate::steps::Step;
use derive_more::Constructor;
use kodept_interpret::operator_desugaring::{
    AccessExpander, BinaryOperatorExpander, UnaryOperatorExpander,
};
use kodept_interpret::semantic_analyzer::ScopeAnalyzer;
use kodept_macros::context::{Context, FileId};
use kodept_macros::error::report::Report;
use std::num::NonZeroU16;
use tracing::info;

#[derive(Constructor)]
pub struct Config {
    pub recursion_depth: NonZeroU16,
}

pub fn run_common_steps<'r>(
    mut ctx: Context<'r>,
    _: &Config,
) -> Result<Context<'r>, Report<FileId>> {
    info!("Step 1: Simplify AST");
    let (_, _, _) = Pipeline
        .define_step((
            AccessExpander::new(),
            BinaryOperatorExpander::new(),
            UnaryOperatorExpander::new(),
        ))
        .apply_with_context(&mut ctx)?;

    info!("Step 2: Split by scopes and resolve symbols");
    let (scopes,) = Pipeline
        .define_step((ScopeAnalyzer::new(),))
        .apply_with_context(&mut ctx)?;
    let _ = scopes.into_inner();
    //
    // info!("Step 3: Infer and check types");
    // Pipeline
    //     .define_step(hlist![TypeChecker::new(
    //         &scopes,
    //         config.recursion_depth,
    //         Witness::fact(a, b, c)
    //     )])
    //     .apply_with_context(ctx)?;

    Ok(ctx)
}
