use crate::cli::utils::{DisplayStyle, Extension};
use clap::{Args, ValueEnum};
use kodept_report::codespan::external::{ColorChoice, Config};
use std::io::{stdin, Read};
use std::path::PathBuf;
use kodept_systems::configs::{LexerImpl, ParserImpl};
use kodept_systems::loader::{Loader, LoadingError};

#[derive(Debug, Args, Clone)]
pub struct ParsingConfig {
    /// Use parallelization when parsing
    #[arg(
        short,
        long,
        hide = !cfg!(feature = "parallel"),
        default_value_t = cfg!(feature = "parallel")
    )]
    pub parallel: bool,
    /// Use specific lexer implementation
    #[arg(default_value = "auto", long)]
    pub lexer: LexerChoice,
    /// Use specific parser implementation
    #[arg(default_value = "auto", long)]
    pub parser: ParserChoice,
    /// Switch to parallel implementation if source file is this large (in KB)
    #[arg(default_value_t = 1024, long = "threshold")]
    pub parallel_threshold: usize,
}

/// Names of all possible lexers
#[derive(Debug, ValueEnum, Clone)]
pub enum LexerChoice {
    Peg,
    ASCII,
    Auto,
}

/// Names of all possible parsers
#[derive(Debug, ValueEnum, Clone)]
pub enum ParserChoice {
    Peg,
    Auto,
}

#[derive(Debug, Args, Clone)]
pub struct DiagnosticConfig {
    /// The display style to use when rendering a diagnostic
    #[arg(ignore_case = true, long = "style", default_value_t = DisplayStyle::Rich)]
    pub style: DisplayStyle,
    /// Add indentation
    #[arg(default_value_t = 4, long)]
    pub tab_width: usize,
    /// Adjust color output settings
    #[arg(short, long, default_value = "auto")]
    pub color: ColorChoice,
    /// Output diagnostics eagerly
    #[arg(long, default_value_t = false)]
    pub eager: bool,
    /// Show all context lines
    #[arg(long = "full-context", default_value_t = false)]
    pub show_full_context_lines: bool,
    /// Disable output of diagnostics to stderr
    #[arg(
    conflicts_with_all = ["style", "tab_width", "color", "eager"],
    long = "disable-diagnostics",
    default_value_t = false
    )]
    pub disable: bool,
}

#[derive(Debug, Args, Clone)]
pub struct LoadingConfig {
    /// Read input from stdin
    #[arg(long = "stdin")]
    read_stdin: bool,
    /// Read input from the specified places
    #[arg(conflicts_with = "read_stdin")]
    input: Vec<PathBuf>,
    /// Use this extension for files
    #[arg(short = 'e', long, default_value = "kd")]
    extension: Extension,
}

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
