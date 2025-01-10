use crate::cli::commands::to_diagnostics;
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::traits::CommandWithSources;
use clap::Args;
use kodept::codespan_settings::{ProvideCollector, Reports};
use kodept::context::Context;
use kodept::loader::Loader;
use kodept::source_files::{SourceFiles, SourceView};
use kodept_ast::syntax_tree::prelude::AST;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::span::CodeHolder;
use kodept_core::Freeze;
use kodept_report::error::report_collector::{ReportCollector, Reporter};
use kodept_report::error::traits::DrainReports;
use std::borrow::Cow;
use std::num::NonZeroU16;
use std::path::Path;
use tracing::debug;

#[derive(Debug, Args, Clone)]
pub struct Execute {
    /// Specifies maximum number of steps while type checking a function
    #[arg(default_value_t = NonZeroU16::new(256).unwrap(), long = "recursion_depth")]
    type_checking_recursion_depth: NonZeroU16,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
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
        match SourceFiles::try_from_sources(loader.into_sources()) {
            Ok(x) => Some(x),
            Err(e) => {
                collector.report((), e);
                None
            }
        }
    }

    fn exec_for_source(&self, source: SourceView, reports: &mut Reports, _: &Path) -> Option<()> {
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
        debug!("Produced AST with node count = {}", ast.node_count());

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
