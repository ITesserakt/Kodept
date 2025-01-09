use crate::cli::commands::{get_output_file, to_diagnostics};
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::traits::CommandWithSources;
use clap::Parser;
use kodept::codespan_settings::{ProvideCollector, Reports};
use kodept::loader::Loader;
use kodept::source_files::{SourceFiles, SourceView};
use kodept_core::Freeze;
use kodept_report::error::report_collector::{ReportCollector, Reporter};
use kodept_report::error::traits::DrainReports;
use std::path::Path;
use kodept::context::Context;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_ast_nodes::file::FileDecl;

#[derive(Parser, Debug, Clone)]
pub struct Graph {
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
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

        let code_holder = || {
            #[cfg(feature = "interning")]
            return kodept_interning::InterningCodeHolder::new(&*source);
            #[cfg(not(feature = "interning"))]
            return kodept::read_code_source::CloningCodeHolder::new(&*source);
        };
        let (tree, accessor) = AST::recursively_build::<FileDecl>(rlt, code_holder());
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

            Some(())
        })
    }
}
