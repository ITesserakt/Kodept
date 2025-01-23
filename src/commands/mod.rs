mod inspect;
mod utils;

use crate::commands::inspect::Inspect;
use clap::Subcommand;
use kodept::report::GlobalReports;
use kodept_frontend::Execution;
use crate::cli::primary::OutputConfig;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Inspect various parts of a compilation process
    Inspect(Inspect),
    // /// Output parsing process files
    // InspectParser(InspectParser),
    // /// Run type checker
    // Execute(Execute),
}

pub trait Command {
    fn exec(self, reports: GlobalReports, config: OutputConfig) -> Execution<()>;
}
