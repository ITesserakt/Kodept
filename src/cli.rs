use std::convert::Infallible;
use std::ffi::OsString;
use std::io::{Read, stdin};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use clap::{Args, Parser, ValueEnum};
use codespan_reporting::term::{ColorArg, Config};
use codespan_reporting::term::termcolor::StandardStream;
use derive_more::Display;
use tracing::Level;

use kodept::codespan_settings::{CodespanSettings, StreamOutput};
use kodept_core::loader::{Loader, LoadingError};

const ABOUT_MESSAGE: &str =
    "Typechecks or interprets passed INPUT using Kodept programming language";

#[derive(Parser, Debug)]
#[command(version, author, about = ABOUT_MESSAGE)]
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
    #[arg(short = 'o', long = "out", default_value = "./")]
    pub output: PathBuf,

    #[command(flatten)]
    pub diagnostic_config: DiagnosticConfig,
    #[command(flatten)]
    pub loading_config: LoadingConfig,
}

#[derive(Debug, Args)]
pub struct DiagnosticConfig {
    /// The display style to use when rendering a diagnostic
    #[arg(ignore_case = true, long = "style", default_value_t = DisplayStyle::Rich)]
    style: DisplayStyle,
    /// Add indentation
    #[arg(default_value_t = 4, long)]
    tab_width: usize,
    /// Adjust color output settings
    #[arg(short, long, default_value = "auto")]
    color: ColorArg,
    /// Enable output of diagnostics to stderr
    #[arg(
        conflicts_with_all = ["style", "tab_width", "color"],
        long = "disable-diagnostics", 
        default_value_t = false
    )]
    disable: bool,
}

#[derive(Debug, Args)]
pub struct LoadingConfig {
    /// Read input from stdin
    #[arg(long = "stdin")]
    read_stdin: bool,
    /// Read input from the specified places
    #[arg(conflicts_with = "read_stdin", required = true)]
    input: Vec<PathBuf>,
    /// Use this extension for files
    #[arg(short = 'e', long, default_value = "kd")]
    extension: Extension,
}

#[derive(Debug, Clone, Display, ValueEnum)]
pub enum DisplayStyle {
    /// Adds code preview
    Rich,
    /// Adds notes
    Medium,
    /// Adds file, line number, severity and message
    Short,
}

#[derive(Clone, Debug)]
pub enum Extension {
    Any,
    Specified(OsString),
}

impl FromStr for Extension {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "*" => Extension::Any,
            _ => Extension::Specified(OsString::from(s)),
        })
    }
}

impl From<DiagnosticConfig> for CodespanSettings {
    fn from(value: DiagnosticConfig) -> Self {
        let config = Config {
            tab_width: value.tab_width,
            display_style: value.style.into(),
            ..Default::default()
        };
        let stream = if !value.disable {
            StreamOutput::Standard(Arc::new(Mutex::new(StandardStream::stderr(value.color.0))))
        } else {
            StreamOutput::NoOp
        };

        Self { config, stream }
    }
}

impl From<DisplayStyle> for codespan_reporting::term::DisplayStyle {
    fn from(value: DisplayStyle) -> Self {
        match value {
            DisplayStyle::Rich => Self::Rich,
            DisplayStyle::Medium => Self::Medium,
            DisplayStyle::Short => Self::Short,
        }
    }
}

impl TryFrom<LoadingConfig> for Loader {
    type Error = LoadingError;

    fn try_from(value: LoadingConfig) -> Result<Self, Self::Error> {
        if value.read_stdin {
            let mut stdin_input = String::new();
            stdin().read_to_string(&mut stdin_input)?;
            Ok(Loader::from_single_snippet(stdin_input))
        } else {
            let builder = Loader::file();
            let builder = match value.extension {
                Extension::Any => builder.with_any_source_extension(),
                Extension::Specified(ext) => builder.with_extension(ext),
            };
            let builder = match value.input.first() {
                None => builder,
                Some(x) => builder.with_starting_path(x),
            };
            builder.build()
        }
    }
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
