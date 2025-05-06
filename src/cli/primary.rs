use crate::cli::configs::DiagnosticConfig;
use crate::commands::Commands;
use clap::{Args, Parser};
use kodept_core::file_name::FileName;
use std::ffi::OsStr;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::PathBuf;
use tracing::Level;

#[derive(Parser, Debug)]
#[command(version, author)]
#[command(propagate_version = true)]
pub struct Kodept {
    #[command(subcommand)]
    pub subcommands: Commands,

    /// Controls how many parallel threads will be created for operations
    #[arg(short = 'p', long, hide = cfg!(not(feature = "parallel")), default_value_t = 0)]
    pub parallelism: usize,
    #[command(flatten, next_help_heading = "Output options")]
    pub output_config: OutputConfig,
    #[command(flatten, next_help_heading = "Diagnostics options")]
    pub diagnostic_config: DiagnosticConfig,
    #[command(flatten, next_help_heading = "Logging options")]
    pub logging: LoggingOptions,
}

#[derive(Debug, Args)]
pub struct OutputConfig {
    /// Write all output to the specified path
    #[arg(short = 'o', long = "out", default_value = "./build", global = true)]
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

impl OutputConfig {
    pub fn create_missing_folders(&self) -> std::io::Result<()> {
        match create_dir_all(&self.output) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn open_file_for_source<Q: AsRef<OsStr> + ?Sized>(
        &self,
        source: &FileName,
        extension: &Q,
    ) -> std::io::Result<File> {
        self.create_missing_folders()?;
        let new_path = source.build_file_path().with_extension(extension.as_ref());
        let name = new_path.file_name().unwrap();
        File::create(self.output.join(name))
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
