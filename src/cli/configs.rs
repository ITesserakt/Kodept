use crate::cli::utils::{DisplayStyle, Extension};
use clap::{Args, ValueEnum};
use derive_more::From;
use kodept::loader::{Loader, LoadingError};
use kodept_parse::lexer::{PegLexer, PestLexer};
use kodept_parse::parser::{PegParser};
use std::io::{stdin, Read};
use std::path::PathBuf;
use kodept_report::codespan::external::ColorArg;

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
    Pest,
    #[cfg(feature = "nom")]
    Nom,
    Auto,
}

/// Names of all possible parsers
#[derive(Debug, ValueEnum, Clone)]
pub enum ParserChoice {
    Peg,
    #[cfg(feature = "nom")]
    Nom,
    Auto,
}

#[derive(Debug, Args)]
pub struct DiagnosticConfig {
    /// The display style to use when rendering a diagnostic
    #[arg(ignore_case = true, long = "style", default_value_t = DisplayStyle::Rich)]
    pub style: DisplayStyle,
    /// Add indentation
    #[arg(default_value_t = 4, long)]
    pub tab_width: usize,
    /// Adjust color output settings
    #[arg(short, long, default_value = "auto")]
    pub color: ColorArg,
    /// Output diagnostics eagerly
    #[arg(long, default_value_t = false)]
    pub eager: bool,
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

#[derive(Debug, Copy, Clone, From)]
pub enum LexerImpl {
    Peg(PegLexer<false>),
    #[cfg(feature = "nom")]
    Nom(kodept_parse::lexer::NomLexer),
    Pest(PestLexer),
}

#[derive(Debug, From)]
pub enum ParserImpl {
    Peg(PegParser<false>),
    #[cfg(feature = "nom")]
    Nom(kodept_parse::parser::NomParser),
}

impl ParsingConfig {
    pub fn get_lexing_backend(&self, source_len: usize) -> LexerImpl {
        const ONE_MB: usize = 1024 * 1024;

        match (
            &self.lexer,
            source_len,
            self.parallel && cfg!(feature = "parallel"),
            cfg!(feature = "trace"),
        ) {
            (LexerChoice::Peg, _, _, false) => PegLexer::<false>::new().into(),
            (LexerChoice::Peg, _, _, true) => {
                panic!("Cannot use peg lexer when parallelization and tracing are enabled")
            }
            (LexerChoice::Pest, _, _, _) => PestLexer::new().into(),
            #[cfg(feature = "nom")]
            (LexerChoice::Nom, _, false, _) => kodept_parse::lexer::NomLexer::new().into(),
            #[cfg(feature = "nom")]
            (LexerChoice::Nom, _, true, _) => panic!("Cannot use nom lexer in parallel context"),
            (LexerChoice::Auto, ..ONE_MB, false, _) => PestLexer::new().into(),
            (LexerChoice::Auto, _, false, true) => PegLexer::<false>::new().into(),
            (LexerChoice::Auto, _, _, false) => PegLexer::<false>::new().into(),
            #[cfg(feature = "nom")]
            (LexerChoice::Auto, _, true, true) => kodept_parse::lexer::NomLexer::new().into(),
            #[cfg(not(feature = "nom"))]
            (LexerChoice::Auto, _, true, true) => PestLexer::new().into()
        }
    }

    pub fn get_parsing_backend(&self) -> ParserImpl {
        match (
            &self.parser,
            self.parallel && cfg!(feature = "parallel"),
            cfg!(feature = "trace"),
        ) {
            (ParserChoice::Peg, _, false) => PegParser::new().into(),
            (ParserChoice::Peg, false, true) => PegParser::new().into(),
            (ParserChoice::Peg, true, true) => {
                panic!("Cannot use peg parser when parallelization and tracing are enabled")
            }
            #[cfg(feature = "nom")]
            (ParserChoice::Nom, _, _) => kodept_parse::parser::NomParser::new().into(),
            (ParserChoice::Auto, _, false) => PegParser::new().into(),
            (ParserChoice::Auto, false, true) => PegParser::new().into(),
            #[cfg(feature = "nom")]
            (ParserChoice::Auto, true, true) => kodept_parse::parser::NomParser::new().into(),
            #[cfg(not(feature = "nom"))]
            (ParserChoice::Auto, true, true) => panic!("Cannot use peg parser when parallelization and tracing are enabled"),
        }
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
