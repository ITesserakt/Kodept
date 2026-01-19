//! Common structures and plugins for cli apps

mod config;
mod plugins;
mod utils;

pub mod prelude {
    pub use super::config::{DiagnosticConfig, LoadingConfig, OutputConfig, ParsingConfig};
    pub use super::plugins::{LogPlugin, ReportsPlugin, ThreadPoolPlugin};
    pub use super::utils::{
        ColorChoice, DisplayStyle, Extension, LexerChoice, LoggingLevel, LoggingOptions,
        ParserChoice,
    };
}
