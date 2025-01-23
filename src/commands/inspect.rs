use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::commands::utils::filesystem::require_output_file;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;
use crate::commands::Command;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept::source::collection::SourceView;
use kodept_core::code_point::CodePoint;
use kodept_report::error::report::{ad_hoc_message, IntoSpannedReportMessage, Label, Severity};
use kodept_report::error::Diagnostic;
use kodept_rlt::prelude::RLT;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use std::path::{Path, PathBuf};
use thiserror::__private::AsDisplay;
use tracing::{error, info};
use kodept_frontend::Execution;

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
    fn exec(self, reports: GlobalReports, output: PathBuf) -> ControlFlow<(), ()> {
        let (sources, reports) = get_all_sources(&self.loading_config, reports)?;
        for source in sources.collect() {
            let rlt = get_rlt(&self.parsing_config, &source, &reports)?;
            if self.export_rlt {
                if let Continue(path) = export_rlt(&source, &output, &rlt) {
                    reports.report(
                        *source.id,
                        ad_hoc_message(|| {
                            Diagnostic::new(Severity::Note)
                                .with_message("Source file parsed into a raw lexeme tree")
                                .with_label(Label::primary("", CodePoint::single_point(0)))
                                .with_note(format!("Exported into {}", path.as_display()))
                        }),
                    );
                }
            }
            
            if self.export_ast {
                
            }
        }
        Continue(())
    }
}

fn export_rlt(source: &SourceView, output: &Path, rlt: &RLT) -> Execution<PathBuf> {
    let filename = source.path().build_file_path();
    let (output_file, file_path) =
        match require_output_file(&output, filename.file_name().unwrap(), "rlt.json") {
            Ok(x) => x,
            Err(e) => {
                error!("Could not open file to output RLT: {e}");
                return Break(());
            }
        };
    if let Err(e) = serde_json::to_writer_pretty(output_file, &rlt) {
        error!("Could serialize RLT into json: {e}");
        return Break(());
    }
    Continue(file_path)
}
