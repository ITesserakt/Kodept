use crate::cli::commands::build_rlt;
use crate::cli::traits::Command;
use clap::Parser;
use kodept::codespan_settings::CodespanSettings;
use kodept::macro_context::DefaultContext;
use kodept::read_code_source::ReadCodeSource;
use kodept::steps::common;
use kodept::steps::common::Config;
use kodept_ast::graph::SyntaxTree;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::ResultTRExt;
use kodept_macros::error::ErrorReported;
use std::num::NonZeroU16;
use tracing::debug;

#[derive(Debug, Parser, Clone)]
pub struct Execute {
    /// Specifies maximum number of steps while type checking a function
    #[arg(default_value_t = NonZeroU16::new(256).unwrap(), long = "recursion_depth")]
    type_checking_recursion_depth: NonZeroU16,
}

impl Command for Execute {
    type Params = ();

    fn exec_for_source(
        &self,
        mut source: ReadCodeSource,
        settings: &mut CodespanSettings,
        _: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        let rlt = build_rlt(&source).or_emit(settings, &source)?;
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, &mut source);
        debug!("Produced AST with node count = {}", tree.node_count());
        let mut context = DefaultContext::new(
            source.with_filename(|_| ReportCollector::new()),
            accessor,
            tree.split().0,
        );
        let config = Config {
            recursion_depth: self.type_checking_recursion_depth,
        };

        common::run_common_steps(&mut context, &config).or_emit(settings, &source)?;
        context.emit_diagnostics(settings, &source);
        Ok(())
    }
}
