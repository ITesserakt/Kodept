use crate::cli::commands::to_diagnostics;
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::traits::CommandWithSources;
use clap::Args;
use kodept::codespan_settings::{ProvideCollector, Reports};
use kodept::loader::Loader;
use kodept::source_files::{SourceFiles, SourceView};
use kodept::steps::common::Config;
use kodept_ast::graph::SyntaxTree;
use kodept_ast::interning::{debug_interning_efficiency, InterningCodeHolder};
use kodept_core::Freeze;
use kodept_macros::context::Context;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::DrainReports;
use std::num::NonZeroU16;
use std::path::Path;
use tracing::debug;

#[derive(Debug, Args, Clone)]
pub struct Execute {
    /// Specifies maximum number of steps while type checking a function
    #[arg(default_value_t = NonZeroU16::new(256).unwrap(), long = "recursion_depth")]
    type_checking_recursion_depth: NonZeroU16,
    #[command(flatten)]
    parsing_config: ParsingConfig,
    #[command(flatten)]
    loading_config: LoadingConfig,
}

impl CommandWithSources for Execute {
    fn build_sources(&self, collector: &mut ReportCollector<()>) -> Option<SourceFiles> {
        let loader: Loader = match self.loading_config.clone().try_into() {
            Ok(x) => x,
            Err(e) => {
                collector.report((), e);
                return None;
            }
        };
        Some(SourceFiles::from_sources(loader.into_sources()))
    }

    fn exec_for_source(&self, source: SourceView, reports: &mut Reports, _: &Path) -> Option<()> {
        let rlt = reports.provide_collector(source.all_files(), |collector| {
            self.parsing_config
                .build_rlt(&source)
                .map_err(to_diagnostics)
                .drain(*source.id, collector)
        })?;

        let code_holder = InterningCodeHolder::new(&*source);
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, code_holder);
        debug_interning_efficiency();
        debug!("Produced AST with node count = {}", tree.node_count());

        reports.provide_collector(source.all_files(), |collector| {
            let mut context = Context {
                ast: tree,
                rlt: accessor,
                collector,
                current_file: Freeze::new(source.describe()),
            };
            let config = Config {
                recursion_depth: self.type_checking_recursion_depth,
            };

            kodept::steps::common::run_common_steps(&mut context, &config)
        })
    }
}
