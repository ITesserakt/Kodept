use crate::cli::commands::{get_output_file, to_diagnostics};
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::traits::CommandWithSources;
use clap::Parser;
use kodept::codespan_settings::{ProvideCollector, Reports};
use kodept::context::Context;
use kodept::loader::Loader;
use kodept::source_files::{SourceFiles, SourceView};
use kodept_ast::syntax_tree::prelude::AST;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::Freeze;
use kodept_report::error::report_collector::{ReportCollector, Reporter};
use kodept_report::error::traits::DrainReports;
use std::borrow::Cow;
use std::path::Path;
use kodept_core::structure::span::CodeHolder;

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
        match SourceFiles::try_from_sources(loader.into_sources()) {
            Ok(x) => Some(x),
            Err(e) => {
                collector.report((), e);
                None
            }
        }
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

        let (ast, rlt) = {
            #[cfg(feature = "interning")]
            {
                let code_holder = kodept_interning::InterningCodeHolder::new(&*source)
                    .map(|it| Cow::Borrowed(it.0));
                AST::recursively_build::<FileDecl>(rlt, code_holder)
            }
            #[cfg(not(feature = "interning"))]
            {
                let code_holder = source.map(|it| Cow::Owned(it.to_string()));
                AST::recursively_build::<FileDecl>(rlt, code_holder)
            }
        };
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
            let context = Context {
                ast,
                rlt,
                collector,
                current_file: Freeze::new(source.describe()),
            };

            Some(())
        })
    }
}
