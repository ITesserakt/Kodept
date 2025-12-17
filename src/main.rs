use crate::cli::primary::Kodept;
use crate::plugins::{LogPlugin, ReportsPlugin, ThreadPoolPlugin};
use crate::profiler::HeapProfilerGuard;
use clap::Parser;
use kodept_frontend::engine::Engine;
use kodept_systems::global::prelude::RegisterReflectionPlugin;
use std::process::ExitCode;

mod cli;
mod commands;
mod plugins;
mod profiler;

fn main() -> ExitCode {
    let _guard = HeapProfilerGuard::install();
    let cli_options = Kodept::parse();
    let mut engine = Engine::new();

    engine
        .add_plugin(RegisterReflectionPlugin)
        .add_plugin(LogPlugin {
            level: cli_options.logging.level(),
            ..Default::default()
        })
        .add_plugin(ThreadPoolPlugin {
            total_threads: cli_options.jobs,
            ..Default::default()
        })
        .add_plugin_if(
            !cli_options.diagnostic_config.disable,
            ReportsPlugin {
                config: cli_options.diagnostic_config,
            },
        );

    match engine.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
