use crate::cli::commands::{build_rlt, get_output_file};
use crate::cli::traits::Command;
use clap::Parser;
use kodept::source_files::SourceView;
use kodept::steps::pipeline::Pipeline;
use kodept::steps::Step;
use kodept_ast::graph::SyntaxTree;
use kodept_macros::context::Context;
use kodept_macros::default::ASTDotFormatter;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::ErrorReported;
use std::path::PathBuf;
use kodept_core::Freeze;

#[derive(Parser, Debug, Clone)]
pub struct Graph;

impl Command for Graph {
    type Params = PathBuf;

    fn exec_for_source(
        &self,
        source: SourceView,
        collector: &mut ReportCollector,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        let Some(rlt) = build_rlt(&source, collector) else {
            return Err(ErrorReported::new());
        };
        
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, &*source);
        let output_file = match get_output_file(&source, params) {
            Ok(x) => x,
            Err(e) => {
                collector.report(*source.id, e);
                return Err(ErrorReported::new());
            }
        };
        let mut context = Context {
            ast: tree,
            rlt: accessor,
            collector,
            current_file: Freeze::new(source.describe()),
        };

        let _: Option<(_,)> = Pipeline
            .define_step((ASTDotFormatter::new(output_file),))
            .apply_with_context(&mut context);
        Ok(())
    }
}
