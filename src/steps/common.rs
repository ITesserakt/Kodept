use crate::macro_context::MacroContext;
use crate::steps::capabilities::BasicCapability;
use crate::steps::pipeline::Pipeline;
use crate::steps::Step;
use derive_more::Constructor;
use kodept_interpret::operator_desugaring::{
    AccessExpander, BinaryOperatorExpander, UnaryOperatorExpander,
};
use kodept_interpret::semantic_analyzer::ScopeAnalyzer;
use kodept_macros::context::Context;
use kodept_macros::default::ASTFormatter;
use kodept_macros::unrecoverable_error::UnrecoverableError;
use std::io::stdout;
use std::num::NonZeroU16;
use tracing::{info, trace_span};

#[derive(Constructor)]
pub struct Config {
    pub recursion_depth: NonZeroU16,
}

pub fn run_common_steps<'a>(
    mut ctx: MacroContext<BasicCapability<'a>>,
    config: &Config,
) -> Result<impl Context + 'a, UnrecoverableError> {
    info!("Step 1: Simplify AST");
    let (a, b, c) = Pipeline
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
    let scopes = scopes.into_inner();
    //
    // info!("Step 3: Infer and check types");
    // Pipeline
    //     .define_step(hlist![TypeChecker::new(
    //         &scopes,
    //         config.recursion_depth,
    //         Witness::fact(a, b, c)
    //     )])
    //     .apply_with_context(ctx)?;

    Ok(ctx.enrich(|_| ()))
}
