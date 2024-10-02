use crate::cli::commands::{get_output_file, to_diagnostics};
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::traits::CommandWithSources;
use clap::Parser;
use kodept::codespan_settings::{ProvideCollector, Reports};
use kodept::loader::Loader;
use kodept::source_files::{SourceFiles, SourceView};
use kodept::steps::pipeline::Pipeline;
use kodept::steps::Step;
use kodept_ast::graph::SyntaxTree;
use kodept_core::Freeze;
use kodept_macros::context::Context;
use kodept_macros::default::ASTDotFormatter;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::DrainReports;
use std::path::Path;
use kodept_ast::interning::InterningCodeHolder;

#[derive(Parser, Debug, Clone)]
pub struct Graph {
    #[command(flatten)]
    parsing_config: ParsingConfig,
    #[command(flatten)]
    loading_config: LoadingConfig,
}

impl CommandWithSources for Graph {
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

    fn exec_for_source(
        &self,
        source: SourceView,
        reports: &mut Reports,
        output: &Path,
    ) -> Option<()> {
        let rlt = reports.provide_collector(source.all_files(), |collector| {
            self.parsing_config
                .build_rlt(&source)
                .map_err(to_diagnostics)
                .drain(*source.id, collector)
        })?;

        let code_holder = InterningCodeHolder::new(&*source);
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, code_holder);
        let output_file = match get_output_file(&source, output) {
            Ok(x) => x,
            Err(e) => {
                reports.provide_collector(source.all_files(), |collector| {
                    collector.report(*source.id, e);
                });
                return None;
            }
        };

        reports.provide_collector(source.all_files(), |collector| {
            let mut context = Context {
                ast: tree,
                rlt: accessor,
                collector,
                current_file: Freeze::new(source.describe()),
            };

            let _: (_,) = Pipeline
                .define_step((ASTDotFormatter::new(output_file),))
                .apply_with_context(&mut context)?;
            Some(())
        })
    }
}
