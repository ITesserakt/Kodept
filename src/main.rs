use crate::cli::make_reports;
use crate::cli::primary::Kodept;
use crate::profiler::HeapProfilerGuard;
use clap::Parser;
use tracing::Level;

mod cli;
mod commands;
mod profiler;

fn init_tracing(level: Level) {
    tracing_subscriber::fmt().with_max_level(level).init();
}

fn init_thread_pool(_parallelism: usize) {
    #[cfg(feature = "parallel")]
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(_parallelism)
            .build_global()
            .expect("Cannot initialize rayon thread pool");
    }
}

fn main() {
    let _guard = HeapProfilerGuard::install();
    let cli_options = Kodept::parse();

    init_tracing(cli_options.logging.level());
    init_thread_pool(cli_options.parallelism);
    let reports = make_reports(cli_options.diagnostic_config);

    let result = cli_options
        .subcommands
        .exec(reports, cli_options.output_config);
    if result.is_break() {
        eprintln!("Compilation finished with errors");
    }
}
