use std::sync::LazyLock;
use clap::Parser;
use kodept_cli::prelude::{DiagnosticConfig, LoggingOptions};

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
    /// Controls how many parallel threads will be created for operations
    #[arg(short = 'j', long, hide = cfg!(not(feature = "parallel")), default_value_t = *DEFAULT_JOBS)]
    pub jobs: usize,
    #[command(flatten, next_help_heading = "Diagnostics options")]
    pub diagnostic_config: DiagnosticConfig,
    #[command(flatten, next_help_heading = "Logging options")]
    pub logging: LoggingOptions,
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
