use std::io::{stdin, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::cli::utils::{DisplayStyle, Extension};
use clap::{Args, ValueEnum};
use codespan_reporting::term::termcolor::StandardStream;
use codespan_reporting::term::{ColorArg, Config};
use derive_more::From;
use kodept::codespan_settings::{CodespanSettings, Reports, StreamOutput};
use kodept::loader::{Loader, LoadingError};
use kodept::read_code_source::ReadCodeSource;
use kodept_core::structure::rlt::RLT;
use kodept_parse::common::{EagerTokensProducer, ErrorAdapter, RLTProducer, TokenProducer};
use kodept_parse::error::ParseErrors;
use kodept_parse::lexer::traits::ToRepresentation;
use kodept_parse::lexer::{NomLexer, PegLexer, PestLexer};
use kodept_parse::parser::{NomParser, PegParser};
use kodept_parse::token_match::PackedTokenMatch;
use kodept_parse::token_stream::PackedTokenStream;
use tracing::debug;

#[derive(Debug, Args, Clone)]
pub struct ParsingConfig {
    /// Do parallelization of source code when parsing
    #[cfg_attr(not(feature = "parallel"), arg(hide = true, default_value_t = false))]
    #[cfg_attr(not(feature = "parallel"), arg(default_value_t = true))]
    #[arg(short, long)]
    parallel: bool,
    /// Use specific lexer implementation
    #[arg(default_value = "auto", long)]
    lexer: LexerChoice,
    /// Use specific parser implementation
    #[arg(default_value = "auto", long)]
    parser: ParserChoice,
    /// Switch to parallel implementation if source file is this large (in KB)
    #[arg(default_value_t = 1024, long = "threshold")]
    parallel_threshold: usize,
}

/// Names of all possible lexers
#[derive(Debug, ValueEnum, Clone)]
pub enum LexerChoice {
    Peg,
    Pest,
    Nom,
    Auto,
}

/// Names of all possible parsers
#[derive(Debug, ValueEnum, Clone)]
pub enum ParserChoice {
    Peg,
    Nom,
    Auto,
}

#[derive(From, Debug, Copy, Clone)]
enum LexerImpl {
    Peg(PegLexer<false>),
    Nom(NomLexer),
    Pest(PestLexer),
}

#[derive(Debug, From)]
enum ParserImpl {
    Peg(PegParser<false>),
    Nom(NomParser),
}

impl LexerImpl {
    fn type_name(&self) -> &'static str {
        match self {
            LexerImpl::Peg(x) => std::any::type_name_of_val(x),
            LexerImpl::Nom(x) => std::any::type_name_of_val(x),
            LexerImpl::Pest(x) => std::any::type_name_of_val(x),
        }
    }
}

impl TokenProducer for LexerImpl {
    type Error<'t> = ParseErrors<&'t str>;

    fn parse_string<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<PackedTokenMatch, Self::Error<'t>> {
        match self {
            LexerImpl::Peg(x) => TokenProducer::parse_string(x, whole_input, position)
                .map_err(|e| e.adapt(whole_input, position)),
            LexerImpl::Nom(x) => TokenProducer::parse_string(x, whole_input, position)
                .map_err(|e| e.adapt(whole_input, position)),
            LexerImpl::Pest(x) => TokenProducer::parse_string(x, whole_input, position)
                .map_err(|e| e.adapt(whole_input, position)),
        }
    }
}

impl EagerTokensProducer for LexerImpl {
    type Error<'t> = ParseErrors<&'t str>;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<PackedTokenMatch>, Self::Error<'t>> {
        match self {
            LexerImpl::Peg(x) => {
                EagerTokensProducer::parse_string(x, input).map_err(|e| e.adapt(input, 0))
            }
            LexerImpl::Nom(_) => unreachable!(),
            LexerImpl::Pest(x) => {
                EagerTokensProducer::parse_string(x, input).map_err(|e| e.adapt(input, 0))
            }
        }
    }
}

impl RLTProducer for ParserImpl {
    type Error<'t> = ParseErrors<&'static str>;

    fn parse_stream<'t>(&self, input: &PackedTokenStream<'t>) -> Result<RLT, Self::Error<'t>> {
        match self {
            ParserImpl::Peg(x) => RLTProducer::parse_stream(x, input)
                .map_err(|e| e.adapt(*input, 0).map(|it| it.representation())),
            ParserImpl::Nom(x) => RLTProducer::parse_stream(x, input)
                .map_err(|e| e.adapt(*input, 0).map(|it| it.representation())),
        }
    }
}

#[derive(Debug, Args, Clone)]
pub struct DiagnosticConfig {
    /// The display style to use when rendering a diagnostic
    #[arg(ignore_case = true, long = "style", default_value_t = DisplayStyle::Rich)]
    style: DisplayStyle,
    /// Add indentation
    #[arg(default_value_t = 4, long)]
    tab_width: usize,
    /// Adjust color output settings
    #[arg(short, long, default_value = "auto")]
    color: ColorArg,
    /// Output diagnostics eagerly
    #[arg(long, default_value_t = false)]
    eager: bool,
    /// Disable output of diagnostics to stderr
    #[arg(
    conflicts_with_all = ["style", "tab_width", "color", "eager"],
    long = "disable-diagnostics",
    default_value_t = false
    )]
    disable: bool,
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
    fn get_lexing_backend(&self, source_len: usize) -> LexerImpl {
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
            (LexerChoice::Nom, _, false, _) => NomLexer::new().into(),
            (LexerChoice::Nom, _, true, _) => panic!("Cannot use nom lexer in parallel context"),
            (LexerChoice::Auto, ..ONE_MB, false, _) => PestLexer::new().into(),
            (LexerChoice::Auto, _, false, true) => PegLexer::<false>::new().into(),
            (LexerChoice::Auto, _, _, false) => PegLexer::<false>::new().into(),
            (LexerChoice::Auto, _, true, true) => NomLexer::new().into(),
        }
    }

    fn get_parsing_backend(&self) -> ParserImpl {
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
            (ParserChoice::Nom, _, _) => NomParser::new().into(),
            (ParserChoice::Auto, _, false) => PegParser::new().into(),
            (ParserChoice::Auto, false, true) => PegParser::new().into(),
            (ParserChoice::Auto, true, true) => NomParser::new().into(),
        }
    }

    pub fn tokenize<'a>(
        &self,
        source: &'a ReadCodeSource,
    ) -> Result<Vec<PackedTokenMatch>, ParseErrors<&'a str>> {
        use kodept_parse::tokenizer::*;

        let backend = self.get_lexing_backend(source.contents().len());

        if cfg!(feature = "parallel") && self.parallel {
            if source.contents().len() > self.parallel_threshold * 1024 {
                debug!(backend = backend.type_name(), "Using parallel lexer");
                #[cfg(feature = "parallel")]
                return ParallelTokenizer::new(source.contents(), backend).try_into_vec();
                #[cfg(not(feature = "parallel"))]
                {
                    unreachable!()
                }
            }
        }
        if matches!(self.lexer, LexerChoice::Nom) {
            LazyTokenizer::new(source.contents(), backend).try_into_vec()
        } else {
            EagerTokenizer::new(source.contents(), backend).try_into_vec()
        }
    }

    pub fn build_rlt<'a>(&self, source: &'a ReadCodeSource) -> Result<RLT, ParseErrors<&'a str>> {
        let tokens = self.tokenize(source)?;
        let stream = PackedTokenStream::new(&tokens);
        debug!(length = tokens.len(), "Produced token stream");

        let backend = self.get_parsing_backend();
        let rlt = backend.parse_stream(&stream)?;

        debug!("Produced RLT with modules count {}", rlt.0 .0.len());
        Ok(rlt)
    }
}

impl From<DiagnosticConfig> for Reports {
    fn from(value: DiagnosticConfig) -> Self {
        let config = Config {
            tab_width: value.tab_width,
            display_style: value.style.into(),
            ..Default::default()
        };
        let stream = if !value.disable {
            StreamOutput::Standard(Arc::new(Mutex::new(StandardStream::stderr(value.color.0))))
        } else {
            StreamOutput::NoOp
        };

        match (value.disable, value.eager) {
            (true, _) => Self::Disabled,
            (false, true) => Self::Eager(CodespanSettings { config, stream }),
            (false, false) => Self::Lazy {
                local_reports: Default::default(),
                global_reports: Default::default(),
                settings: CodespanSettings { config, stream }
            },
        }
    }
}

impl TryFrom<LoadingConfig> for Loader {
    type Error = LoadingError;

    fn try_from(value: LoadingConfig) -> Result<Self, Self::Error> {
        if value.read_stdin {
            let mut stdin_input = String::new();
            stdin().read_to_string(&mut stdin_input)?;
            Ok(Loader::from_single_snippet(stdin_input))
        } else {
            let builder = Loader::file();
            let builder = match value.extension {
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
