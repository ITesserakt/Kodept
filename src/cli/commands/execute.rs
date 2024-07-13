use crate::cli::commands::{build_rlt};
use crate::cli::traits::Command;
use clap::Parser;
use kodept::codespan_settings::CodespanSettings;
use kodept::macro_context::DefaultContext;
use kodept::read_code_source::ReadCodeSource;
use kodept::steps::common;
use kodept::steps::common::Config;
use kodept_ast::ast_builder::ASTBuilder;
use kodept_macros::error::ErrorReported;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::ResultTRExt;
use crate::cli::configs::CompilationConfig;

#[derive(Debug, Parser, Clone)]
pub struct Execute;

impl Command for Execute {
    type Params = CompilationConfig;

    fn exec_for_source(
        &self,
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        let rlt = build_rlt(&source).or_emit(settings, &source)?;
        let (tree, accessor) = ASTBuilder.recursive_build(&rlt.0, &source);
        let mut context = DefaultContext::new(
            source.with_filename(|_| ReportCollector::new()),
            accessor,
            tree.build(),
        );
        let config = Config {
            recursion_depth: params.type_checking_recursion_depth,
        };
        
        common::run_common_steps(&mut context, &config).or_emit(settings, &source)?;
        context.emit_diagnostics(settings, &source);
        Ok(())
    }
}
