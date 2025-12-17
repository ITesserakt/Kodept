use clap::ValueEnum;
use std::convert::Infallible;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

impl From<DisplayStyle> for kodept_report::codespan::external::DisplayStyle {
    fn from(value: DisplayStyle) -> Self {
        match value {
            DisplayStyle::Rich => Self::Rich,
            DisplayStyle::Medium => Self::Medium,
            DisplayStyle::Short => Self::Short,
        }
    }
}
