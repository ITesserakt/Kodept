mod check;
mod inspect;

use crate::cli::primary::OutputConfig;
use crate::commands::check::Check;
use crate::commands::inspect::Inspect;
use bevy_ecs::prelude::*;
use clap::Subcommand;
use kodept_frontend::engine::utils::Timings;
use kodept_frontend::engine::{Engine, Plugin, SubEngine, reporter};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect various parts of a compilation process
    Inspect(Inspect),
    /// Apply lints and type check input
    Check(Check),
    // /// Output parsing process files
    // InspectParser(InspectParser),
    // /// Run type checker
    // Execute(Execute),
}

impl Plugin for Commands {
    fn build(self, engine: &mut Engine) {
        match self {
            Commands::Inspect(x) => x.build(engine),
            Commands::Check(x) => x.build(engine),
        }
    }
}

fn inject_common_resources(
    InMut(engine): InMut<SubEngine>,
    timings: Option<Res<Timings>>,
    report_settings: Option<Res<reporter::Settings>>,
    output_config: Res<OutputConfig>,
) {
    if timings.is_some() {
        engine.init_resource::<Timings>();
    }
    if let Some(report_settings) = report_settings {
        engine.insert_resource(report_settings.clone())
    }
    engine.insert_resource(output_config.clone())
}
