use std::path::PathBuf;

use clap::{Args, Parser};
use tracing::Level;
use kodept_frontend::external::Resource;
use crate::cli::commands::Commands;
use crate::cli::configs::DiagnosticConfig;

const ABOUT_MESSAGE: &str =
    "Typechecks or interprets passed INPUT using Kodept programming language";

#[derive(Parser, Debug)]
#[command(version, author, about = ABOUT_MESSAGE)]
#[command(propagate_version = true)]
pub struct Kodept {
    /// Write all output to specified path
    #[arg(short = 'o', long = "out", default_value = "./build", global = true)]
    pub(super) output: PathBuf,

    #[command(subcommand)]
    pub(super) subcommands: Commands,

    #[command(flatten, next_help_heading = "Diagnostics options")]
    pub(super) diagnostic_config: DiagnosticConfig,
    #[command(flatten, next_help_heading = "Logging options")]
    pub(super) logging: LoggingOptions,
}

#[derive(Debug, Args, Resource)]
#[group(required = false, multiple = false)]
pub struct LoggingOptions {
    /// Enable debugging output
    #[arg(short, long)]
    debug: bool,
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Specify logger output level explicitly
    #[arg(
        short,
        long,
        ignore_case = true,
        default_value = "info",
        env = "RUST_LOG"
    )]
    severity: Level,
}

impl LoggingOptions {
    pub fn level(&self) -> Level {
        self
            .debug
            .then_some(Level::DEBUG)
            .or(self.verbose.then_some(Level::DEBUG))
            .unwrap_or(self.severity)
    }
}

#[cfg(test)]
#[test]
fn verify_cli() {
    use clap::CommandFactory;

    Kodept::command().debug_assert();
}
