use crate::cli::utils::{DisplayStyle, Extension};
use clap::{Args, ValueEnum};
use kodept_report::codespan::external::{ColorChoice, Config};
use std::io::{stdin, Read};
use std::path::PathBuf;
use kodept_cli::{DiagnosticConfig, Extension, LexerChoice, LoadingConfig, ParserChoice, ParsingConfig};
use kodept_cli::prelude::{DiagnosticConfig, LoadingConfig, ParsingConfig};
use kodept_systems::configs::{LexerImpl, ParserImpl};
use kodept_systems::loader::{Loader, LoadingError};

impl ParsingConfig {
    pub fn get_lexing_backend(&self, source: &str) -> LexerImpl {
        match (
            &self.lexer,
            source.len(),
            self.parallel && cfg!(feature = "parallel"),
            false,
        ) {
            (LexerChoice::Peg, _, _, false) => LexerImpl::peg(),
            (LexerChoice::Peg, _, _, true) => {
                panic!("Cannot use peg lexer when parallelization and tracing are enabled")
            }
            (LexerChoice::ASCII, _, _, _) if source.is_ascii() => LexerImpl::ascii(),
            (LexerChoice::ASCII, _, _, _) => panic!("Cannot use ascii lexer for non-ascii inputs"),
            (LexerChoice::Auto, _, _, _) if source.is_ascii() => LexerImpl::ascii(),
            (LexerChoice::Auto, _, false, true) => LexerImpl::peg(),
            (LexerChoice::Auto, _, _, false) => LexerImpl::peg(),
            (LexerChoice::Auto, _, _, true) => panic!("Cannot determine lexer for non-ascii input and tracing enabled")
        }
    }

    pub fn get_parsing_backend(&self) -> ParserImpl {
        match (
            &self.parser,
            self.parallel && cfg!(feature = "parallel"),
            false,
        ) {
            (ParserChoice::Peg, _, false) => ParserImpl::peg(),
            (ParserChoice::Peg, false, true) => ParserImpl::peg(),
            (ParserChoice::Peg | ParserChoice::Auto, true, true) => {
                panic!("Cannot use peg parser when parallelization and tracing are enabled")
            }
            (ParserChoice::Auto, _, false) => ParserImpl::peg(),
            (ParserChoice::Auto, false, true) => ParserImpl::peg(),
        }
    }
}

impl From<DiagnosticConfig> for Config {
    fn from(value: DiagnosticConfig) -> Self {
        let mut this = Self {
            display_style: value.style.into(),
            tab_width: value.tab_width,
            ..Default::default()
        };
        if value.show_full_context_lines {
            this.start_context_lines = usize::MAX;
            this.end_context_lines = usize::MAX;
        }
        this
    }
}

impl TryFrom<&LoadingConfig> for Loader {
    type Error = LoadingError;

    fn try_from(value: &LoadingConfig) -> Result<Self, Self::Error> {
        if value.read_stdin {
            let mut stdin_input = String::new();
            stdin().read_to_string(&mut stdin_input)?;
            Ok(Loader::from_single_snippet(stdin_input))
        } else {
            let builder = Loader::file();
            let builder = match &value.extension {
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
