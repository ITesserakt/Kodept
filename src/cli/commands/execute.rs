use crate::cli::commands::to_diagnostics;
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::traits::CommandWithSources;
use clap::Args;
use kodept::loader::Loader;
use kodept::report::{GlobalReports, Reports};
use kodept::source::collection::{SourceView, Sources};
use kodept::steps::common::Config;
use kodept_ast::graph::SyntaxTree;
use kodept_ast::interning::{debug_interning_efficiency, InterningCodeHolder};
use kodept_frontend::prelude::ExtractReports;
use kodept_frontend::Execution;
use kodept_interpret::macros::Context;
use std::num::NonZeroU16;
use std::ops::ControlFlow::{Break, Continue};
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
    fn build_sources(&self, reports: &GlobalReports) -> Execution<Sources> {
        let loader: Loader = (&self.loading_config)
            .try_into()
            .extract_reports_global(reports)?;
        let sources = loader.into_sources().extract_reports_global(reports)?;
        let mut storage = Sources::new();
        for source in sources {
            storage.insert(source).extract_reports_global(reports);
        }
        Continue(storage)
    }

    fn exec_for_source(
        &self,
        source: SourceView,
        reports: &Reports,
        _: &Path,
    ) -> Execution<()> {
        let rlt = self
            .parsing_config
            .build_rlt(&source)
            .map_err(|e| to_diagnostics(e).extract_reports(*source.id, reports))
            .map_or(Break(()), Continue)?;

        let code_holder = InterningCodeHolder::new(&*source);
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, code_holder);
        debug_interning_efficiency();
        debug!("Produced AST with node count = {}", tree.node_count());

        let sink = {
            let reports = reports.clone();
            move |r| reports.insert(r)
        };
        let mut context = Context::new(tree, accessor, sink, source.describe());
        let config = Config {
            recursion_depth: self.type_checking_recursion_depth,
        };

        kodept::steps::common::run_common_steps(&mut context, &config)
    }
}
