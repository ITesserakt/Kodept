//! Common structures and plugins for cli apps

mod config;
mod utils;
mod plugins;

pub mod prelude {
    pub use super::config::{DiagnosticConfig, LoadingConfig, OutputConfig, ParsingConfig};
    pub use super::utils::{
        ColorChoice, DisplayStyle, Extension, LexerChoice, LoggingLevel, LoggingOptions,
        ParserChoice,
    };
    pub use super::plugins::{LogPlugin, ReportsPlugin, ThreadPoolPlugin};
}
