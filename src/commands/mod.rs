mod inspect;
mod utils;
mod typecheck;

use crate::commands::inspect::Inspect;
use clap::Subcommand;
use kodept::report::GlobalReports;
use kodept_frontend::Execution;
use crate::cli::primary::OutputConfig;
use crate::commands::typecheck::TypeCheck;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect various parts of a compilation process
    Inspect(Inspect),
    TypeCheck(TypeCheck),
    // /// Output parsing process files
    // InspectParser(InspectParser),
    // /// Run type checker
    // Execute(Execute),
}

trait Command {
    fn exec(self, reports: GlobalReports, config: OutputConfig) -> Execution<()>;
}

impl Commands {
    pub fn exec(self, reports: GlobalReports, config: OutputConfig) -> Execution<()> {
        match self {
            Commands::Inspect(x) => x.exec(reports, config),
            Commands::TypeCheck(x) => x.exec(reports, config)
        }
    }
}
