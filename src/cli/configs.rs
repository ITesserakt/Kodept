use std::io::{Read, stdin};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::Args;
use codespan_reporting::term::{ColorArg, Config};
use codespan_reporting::term::termcolor::StandardStream;

use kodept::codespan_settings::{CodespanSettings, StreamOutput};
use kodept_core::loader::{Loader, LoadingError};

use crate::cli::utils::{DisplayStyle, Extension};

#[derive(Debug, Args)]
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
    /// Enable output of diagnostics to stderr
    #[arg(
    conflicts_with_all = ["style", "tab_width", "color"],
    long = "disable-diagnostics",
    default_value_t = false
    )]
    disable: bool,
}

#[derive(Debug, Args)]
pub struct LoadingConfig {
    /// Read input from stdin
    #[arg(long = "stdin", global = true)]
    read_stdin: bool,
    /// Read input from the specified places
    #[arg(conflicts_with = "read_stdin", global = true)]
    input: Vec<PathBuf>,
    /// Use this extension for files
    #[arg(short = 'e', long, default_value = "kd")]
    extension: Extension,
}

impl From<DiagnosticConfig> for CodespanSettings {
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

        Self { config, stream }
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
