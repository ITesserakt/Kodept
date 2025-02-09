use crate::cli::commands::{get_output_file, to_diagnostics};
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::traits::CommandWithSources;
use clap::Parser;
use kodept::loader::Loader;
use kodept::report::{GlobalReports, Reports};
use kodept::source::collection::{SourceView, Sources};
use kodept::steps::pipeline::Pipeline;
use kodept::steps::Step;
use kodept_ast::graph::SyntaxTree;
use kodept_ast::interning::InterningCodeHolder;
use kodept_frontend::prelude::ExtractReports;
use kodept_frontend::Execution;
use kodept_interpret::dot_formatter::ASTDotFormatter;
use kodept_interpret::macros::Context;
use std::ops::ControlFlow::Continue;
use std::path::Path;

#[derive(Parser, Debug, Clone)]
pub struct Graph {
    #[command(flatten)]
    parsing_config: ParsingConfig,
    #[command(flatten)]
    loading_config: LoadingConfig,
}

impl CommandWithSources for Graph {
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
        output: &Path,
    ) -> Execution<()> {
        let rlt = self
            .parsing_config
            .build_rlt(&source)
            .map_err(|e| to_diagnostics(e).extract_reports(*source.id, reports))
            .map_or(Execution::Break(()), Continue)?;

        let code_holder = InterningCodeHolder::new(&*source);
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, code_holder);
        let output_file = get_output_file(&source, output).extract_reports(*source.id, reports)?;

        let sink = {
            let reports = reports.clone();
            move |r| reports.insert(r)
        };
        let mut context = Context::new(tree, accessor, sink, source.describe());

        let (_,) = Pipeline
            .define_step((ASTDotFormatter::new(output_file),))
            .apply_with_context(&mut context)?;
        Continue(())
    }
}
