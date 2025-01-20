use clap::Parser;
use tracing::Level;
use cli::common::Kodept;
use kodept::codespan_settings::{ConsumeCollector, Reports};
use kodept::profiler::HeapProfiler;
use kodept::source_files::GlobalReports;

mod cli;

type WideError = anyhow::Error;

fn init_tracing(level: Level) {
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
}

fn init_thread_pool(parallelism: usize) -> Result<(), WideError> {
    #[cfg(feature = "parallel")]
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .build_global()?
    }
    Ok(())
}

fn main() -> Result<(), WideError> {
    let mut lock = HeapProfiler::install();
    lock.consume_on_ctrlc();

    let cli_arguments: Kodept = Kodept::parse();
    init_tracing(cli_arguments.level());
    init_thread_pool(cli_arguments.parallelism)?;

    let reports: Reports = cli_arguments.diagnostic_config.into();
    let result = cli_arguments
        .subcommands
        .execute(cli_arguments.output, reports.clone());
    reports.consume(&GlobalReports);

    Ok(result?)
}
