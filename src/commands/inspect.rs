use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::utils::build_ast::build_ast;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;
use crate::commands::Command;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept::source::collection::SourceView;
use kodept_core::code_point::CodePoint;
use kodept_frontend::Execution;
use kodept_report::error::report::{ad_hoc_message, Label, Severity};
use kodept_report::error::Diagnostic;
use kodept_rlt::prelude::RLT;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use tracing::error;
use kodept_ast::syntax_tree::prelude::AST;

#[derive(Parser, Debug, Clone)]
pub struct Inspect {
    /// Export raw lexeme tree in .json format into a file
    #[arg(short = 'r', action)]
    export_rlt: bool,
    /// Export abstract syntax tree in .dot format into a file
    #[arg(short = 'a', action)]
    export_ast: bool,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
}

impl Command for Inspect {
    fn exec(self, reports: GlobalReports, config: OutputConfig) -> ControlFlow<(), ()> {
        let (sources, reports) = get_all_sources(&self.loading_config, reports)?;
        for source in sources.collect() {
            let rlt = get_rlt(&self.parsing_config, &source, &reports)?;
            if self.export_rlt && export_rlt(&source, &config, &rlt).is_continue() {
                let message = ad_hoc_message(|| {
                    Diagnostic::new(Severity::Note)
                        .with_message("Source file parsed into a raw lexeme tree")
                        .with_label(Label::primary("", CodePoint::single_point(0)))
                });
                reports.report(*source.id, message);
            }

            let (mut ast, _) = build_ast(&source, rlt);
            if self.export_ast && export_ast(&source, &config, &mut ast).is_continue() {
                 let message = ad_hoc_message(|| {
                     Diagnostic::new(Severity::Note)
                         .with_message("Got abstract syntax tree of source file")
                         .with_label(Label::primary("", CodePoint::single_point(0)))
                 });
                reports.report(*source.id, message);
            }
        }
        Continue(())
    }
}

fn export_rlt(source: &SourceView, config: &OutputConfig, rlt: &RLT) -> Execution<()> {
    let output_file = match config.open_file_for_source(source.path(), "rlt.json") {
        Ok(x) => x,
        Err(e) => {
            error!("Could not open file to output RLT: {e}");
            return Break(());
        }
    };
    if let Err(e) = serde_json::to_writer_pretty(output_file, &rlt) {
        error!("Could not serialize RLT into json: {e}");
        return Break(());
    }
    Continue(())
}

fn export_ast(source: &SourceView, config: &OutputConfig, ast: &mut AST) -> Execution<()> {
    let output_file = match config.open_file_for_source(source.path(), "dot") {
        Ok(x) => x,
        Err(e) => {
            error!("Could not open file to output RLT: {e}");
            return Break(());
        }
    };
    if let Err(e) = ast.export_dot(output_file) {
        error!("Could not export AST into .dot: {e}");
        return Break(());
    }
    Continue(())
}
