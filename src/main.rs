use crate::cli::init_reports;
use crate::cli::primary::Kodept;
use crate::profiler::HeapProfilerGuard;
use clap::Parser;
use kodept_frontend::engine::Engine;
use std::process::ExitCode;
use tracing::Level;

mod cli;
mod commands;
mod profiler;

fn init_tracing(level: Level) -> impl FnOnce(&mut Engine) {
    move |_| {
        tracing_subscriber::fmt()
            .with_thread_names(true)
            .with_max_level(level)
            .init()
    }
}

fn init_thread_pool(parallelism: usize) -> impl FnOnce(&mut Engine) {
    move |engine| {
        let max_total_threads = (parallelism / 2).max(1);
        engine.add_plugin(kodept_frontend::engine::utils::TaskPoolPlugin {
            task_pool_options: kodept_frontend::engine::utils::TaskPoolOptions {
                max_total_threads,
                min_total_threads: 1,
                ..Default::default()
            },
        });
        // TODO: combine bevy's thread pool with rayon's one
        //       Maybe `Forte`? (https://github.com/NthTensor/Forte)
        #[cfg(feature = "parallel")]
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism / 2)
            .build_global()
            .expect("Cannot initialize rayon thread pool");
    }
}

fn main() -> ExitCode {
    let _guard = HeapProfilerGuard::install();
    let cli_options = Kodept::parse();
    let mut engine = Engine::new();

    engine.add_plugin(init_tracing(cli_options.logging.level()));
    engine.add_plugin(init_thread_pool(cli_options.jobs));
    engine.add_plugin(init_reports(cli_options.diagnostic_config));
    engine.add_plugin(cli_options.subcommands);

    match engine.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
