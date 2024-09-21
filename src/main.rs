use clap::Parser;
use cli::common::Kodept;
use kodept::codespan_settings::{ConsumeCollector, Reports};
use kodept::profiler::HeapProfiler;
use kodept::source_files::GlobalReports;

mod cli;

type WideError = anyhow::Error;

fn main() -> Result<(), WideError> {
    let mut lock = HeapProfiler::install();
    lock.consume_on_ctrlc();

    let cli_arguments: Kodept = Kodept::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli_arguments.level())
        .init();

    let reports: Reports = cli_arguments.diagnostic_config.into();
    let result = cli_arguments
        .subcommands
        .execute(cli_arguments.output, reports.clone());
    reports.consume(&GlobalReports);

    Ok(result?)
}
