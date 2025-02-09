use std::ops::ControlFlow;
use std::process::ExitCode;
use clap::Parser;
use tracing::Level;
use cli::common::Kodept;
use kodept::profiler::HeapProfiler;

mod cli;

fn init_tracing(level: Level) {
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
}

fn init_thread_pool(_parallelism: usize) {
    #[cfg(feature = "parallel")]
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(_parallelism)
            .build_global()
            .expect("Cannot init thread pool");
    }
}

fn main() -> ExitCode {
    let mut lock = HeapProfiler::install();
    lock.consume_on_ctrlc();

    let cli_arguments: Kodept = Kodept::parse();
    init_tracing(cli_arguments.level());
    init_thread_pool(cli_arguments.parallelism);

    let reports = cli_arguments.diagnostic_config.make_reports();
    let result = cli_arguments
        .subcommands
        .execute(cli_arguments.output, reports);
    
    if let ControlFlow::Break(()) = result {
        eprintln!("Compilation finished with errors");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
