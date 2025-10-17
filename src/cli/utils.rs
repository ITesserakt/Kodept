use clap::ValueEnum;
use std::convert::Infallible;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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

impl From<DisplayStyle> for kodept_report::codespan::external::DisplayStyle {
    fn from(value: DisplayStyle) -> Self {
        match value {
            DisplayStyle::Rich => Self::Rich,
            DisplayStyle::Medium => Self::Medium,
            DisplayStyle::Short => Self::Short,
        }
    }
}
