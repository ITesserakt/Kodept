use crate::cli::configs::DiagnosticConfig;
use crate::commands::Commands;
use clap::{Args, Parser};
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::Level;

static DEFAULT_JOBS: LazyLock<usize> = LazyLock::new(|| {
    #[cfg(feature = "parallel")]
    return std::thread::available_parallelism().unwrap().get();
    #[cfg(not(feature = "parallel"))]
    0
});

#[derive(Parser, Debug)]
#[command(version, author)]
#[command(propagate_version = true)]
pub struct Kodept {
    #[command(subcommand)]
    pub subcommands: Commands,

    /// Controls how many parallel threads will be created for operations
    #[arg(short = 'j', long, hide = cfg!(not(feature = "parallel")), default_value_t = *DEFAULT_JOBS)]
    pub jobs: usize,
    #[command(flatten, next_help_heading = "Diagnostics options")]
    pub diagnostic_config: DiagnosticConfig,
    #[command(flatten, next_help_heading = "Logging options")]
    pub logging: LoggingOptions,
}

#[derive(Debug, Args)]
pub struct OutputConfig {
    /// Write all output to the specified path
    #[arg(short = 'o', long = "out", default_value = "./build")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
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
        self.debug
            .then_some(Level::DEBUG)
            .or(self.verbose.then_some(Level::TRACE))
            .unwrap_or(self.severity)
    }
}

#[cfg(test)]
mod test {
    use crate::cli::primary::Kodept;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;

        Kodept::command().debug_assert();
    }
}
