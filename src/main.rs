use crate::cli::common::LoggingOptions;
use crate::cli::CliArgumentsPlugin;
use bevy_ecs::prelude::Res;
use kodept::codespan_settings::ReportsPlugin;
use kodept::utils::profiler::HeapProfilerPlugin;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::ExitEvent;

mod actions;
mod cli;

fn init_logging(cli_args: Res<LoggingOptions>) {
    tracing_subscriber::fmt()
        .with_max_level(cli_args.level())
        .init();
}

fn main() {
    let mut frontend = Frontend::new();

    frontend
        .add_event::<ExitEvent>()
        .add_plugin(ReportsPlugin)
        .add_plugin(HeapProfilerPlugin)
        .add_plugin(CliArgumentsPlugin);

    frontend.on_startup(init_logging);

    frontend.run();
}
