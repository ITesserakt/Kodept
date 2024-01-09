use std::convert::Infallible;
use std::ffi::OsString;
use std::str::FromStr;

use clap::ValueEnum;
use derive_more::Display;

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

impl From<DisplayStyle> for codespan_reporting::term::DisplayStyle {
    fn from(value: DisplayStyle) -> Self {
        match value {
            DisplayStyle::Rich => Self::Rich,
            DisplayStyle::Medium => Self::Medium,
            DisplayStyle::Short => Self::Short,
        }
    }
}
