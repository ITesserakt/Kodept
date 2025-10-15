mod check;
mod inspect;
mod utils;

use crate::cli::primary::OutputConfig;
use crate::commands::check::Check;
use crate::commands::inspect::Inspect;
use clap::Subcommand;
use kodept::report::GlobalReports;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_frontend::Execution;

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
            Commands::Check(_) => {}
        }
    }
}

trait CommandV2 {
    fn build(self, engine: &mut Engine, config: OutputConfig);
}

trait Command {
    fn exec(self, reports: GlobalReports, config: OutputConfig) -> Execution<()>;
}
