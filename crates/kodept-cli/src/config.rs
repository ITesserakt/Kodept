use crate::utils::{ColorChoice, DisplayStyle, Extension, LexerChoice, ParserChoice};
use clap::Args;
use std::path::PathBuf;

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
    pub read_stdin: bool,
    /// Read input from the specified places
    #[arg(conflicts_with = "read_stdin")]
    pub input: Vec<PathBuf>,
    /// Use this extension for files
    #[arg(short = 'e', long, default_value = "kd")]
    pub extension: Extension,
}

#[derive(Debug, Args, Clone)]
pub struct OutputConfig {
    /// Write all output to the specified path
    #[arg(short = 'o', long = "out", default_value = "./build")]
    pub output: PathBuf,
}
