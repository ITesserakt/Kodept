use crate::cli::primary::Kodept;
use crate::plugins::{LogPlugin, ReportsPlugin, ThreadPoolPlugin};
use crate::profiler::HeapProfilerGuard;
use clap::Parser;
use kodept_frontend::engine::Engine;
use std::process::ExitCode;
use kodept_systems::lint::DefaultLintsPlugin;

mod cli;
mod commands;
mod plugins;
mod profiler;

fn main() -> ExitCode {
    let _guard = HeapProfilerGuard::install();
    let cli_options = Kodept::parse();
    let mut engine = Engine::new();

    engine
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
                colored: cli_options.diagnostic_config.color,
                eager: cli_options.diagnostic_config.eager,
                config: cli_options.diagnostic_config.into(),
                ..Default::default()
            },
        )
        .add_plugin(cli_options.subcommands)
        .add_plugin(DefaultLintsPlugin);

    match engine.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
