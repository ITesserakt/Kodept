use std::ops::ControlFlow::Continue;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept_frontend::Execution;
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::Command;
use crate::commands::utils::build_ast::build_ast;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;

#[derive(Debug, Parser)]
pub struct TypeCheck {
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig
}

impl Command for TypeCheck {
    fn exec(self, reports: GlobalReports, _: OutputConfig) -> Execution<()> {
        let (sources, reports) = get_all_sources(&self.loading_config, reports)?;
        for source in sources.collect() {
            let rlt = get_rlt(&self.parsing_config, &source, &reports)?;
            let (ast, syntax) = build_ast(&source, rlt);
        }
        Continue(())
    }
}