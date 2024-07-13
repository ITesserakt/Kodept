use std::path::PathBuf;

use clap::Parser;
use tracing::Level;

use crate::cli::commands::Commands;
use crate::cli::configs::{CompilationConfig, DiagnosticConfig, LoadingConfig};

const ABOUT_MESSAGE: &str =
    "Typechecks or interprets passed INPUT using Kodept programming language";

#[derive(Parser, Debug, Clone)]
#[command(version, author, about = ABOUT_MESSAGE)]
#[command(propagate_version = true)]
pub struct Kodept {
    /// Enable debugging output
    #[arg(short, long)]
    debug: bool,
    /// Enable verbose output
    #[arg(short, long, conflicts_with = "debug")]
    verbose: bool,
    /// Set logger output level
    #[arg(
    short = 's',
    long = "severity",
    ignore_case = true,
    default_value = "info",
    env = "RUST_LOG",
    conflicts_with_all = ["debug", "verbose"]
    )]
    verbosity: Level,
    /// Write output to specified path
    #[arg(short = 'o', long = "out", default_value = "./build")]
    pub output: PathBuf,

    #[command(flatten)]
    pub diagnostic_config: DiagnosticConfig,
    #[command(flatten)]
    pub loading_config: LoadingConfig,
    #[command(flatten)]
    pub compilation_config: CompilationConfig,
    #[command(subcommand)]
    pub subcommands: Commands,
}

impl Kodept {
    pub fn level(&self) -> Level {
        if self.debug {
            Level::DEBUG
        } else if self.verbose {
            Level::TRACE
        } else {
            self.verbosity
        }
    }
}
