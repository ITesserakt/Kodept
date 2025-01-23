use clap::Parser;
use tracing::Level;
use crate::cli::make_reports;
use crate::cli::primary::Kodept;
use crate::profiler::HeapProfilerGuard;

mod cli;
mod commands;
mod profiler;

fn init_tracing(level: Level) {
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
}

fn main() {
    let _guard = HeapProfilerGuard::install();
    let cli_options = Kodept::parse();
    
    init_tracing(cli_options.logging.level());
    let reports = make_reports(cli_options.diagnostic_config);

    let result = cli_options.subcommands.exec(reports, cli_options.output_config);
    if result.is_break() {
        eprintln!("Compilation finished with errors");
    }
}