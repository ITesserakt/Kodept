use clap::{Args, ValueEnum};
use std::convert::Infallible;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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
    severity: LoggingLevel,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum LoggingLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    Always,
    Never,
    Auto,
}

/// Names of all possible lexers
#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum LexerChoice {
    Peg,
    ASCII,
    Auto,
}

/// Names of all possible parsers
#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum ParserChoice {
    Peg,
    Auto,
}

#[derive(Debug, Clone, ValueEnum)]
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

impl LoggingOptions {
    pub fn level(&self) -> LoggingLevel {
        self.debug
            .then_some(LoggingLevel::Debug)
            .or(self.verbose.then_some(LoggingLevel::Trace))
            .unwrap_or(self.severity)
    }
}

impl Display for DisplayStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayStyle::Rich => write!(f, "rich"),
            DisplayStyle::Medium => write!(f, "medium"),
            DisplayStyle::Short => write!(f, "short"),
        }
    }
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
