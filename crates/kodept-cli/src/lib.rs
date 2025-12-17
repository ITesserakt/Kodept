//! Common structures for cli apps

mod config;
mod utils;

pub mod prelude {
    pub use super::config::{DiagnosticConfig, LoadingConfig, OutputConfig, ParsingConfig};
    pub use super::utils::{
        ColorChoice, DisplayStyle, Extension, LexerChoice, LoggingLevel, LoggingOptions,
        ParserChoice,
    };
}
