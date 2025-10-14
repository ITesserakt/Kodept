use crate::cli::init_reports;
use crate::cli::primary::Kodept;
use crate::profiler::HeapProfilerGuard;
use clap::Parser;
use tracing::Level;
use kodept_frontend::engine::Engine;

mod cli;
mod commands;
mod profiler;
mod phases;

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
    let mut engine = Engine::new();

    #[cfg(feature = "parallel")]
    engine.add_plugin(kodept_frontend::engine::utils::TaskPoolPlugin::default());

    init_tracing(cli_options.logging.level());
    init_thread_pool(cli_options.jobs);
    init_reports(cli_options.diagnostic_config, &mut engine);

    cli_options.subcommands.build(&mut engine, cli_options.output_config);
    engine.run();
}
