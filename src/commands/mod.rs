mod check;
mod inspect;

use std::io::{stdin, Read};
use crate::commands::check::Check;
use crate::commands::inspect::Inspect;
use clap::Subcommand;
use kodept_cli::prelude::{Extension, LoadingConfig};
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::loader::{Loader, LoadingError};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inspect various parts of a compilation process
    Inspect(Inspect),
    /// Apply lints and type check input
    Check(Check),
    // /// Output parsing process files
    // InspectParser(InspectParser),
    // /// Run type checker
    // Execute(Execute),
}

impl Plugin for Commands {
    fn build(self, engine: &mut Engine) {
        match self {
            Commands::Inspect(x) => x.build(engine),
            Commands::Check(x) => x.build(engine),
        }
    }
}

#[derive(Debug)]
struct Convert(LoadingConfig);

impl TryFrom<&Convert> for Loader {
    type Error = LoadingError;

    fn try_from(value: &Convert) -> Result<Self, Self::Error> {
        if value.0.read_stdin {
            let mut stdin_input = String::new();
            stdin().read_to_string(&mut stdin_input)?;
            Ok(Loader::from_single_snippet(stdin_input))
        } else {
            let builder = Loader::file();
            let builder = match &value.0.extension {
                Extension::Any => builder.with_any_source_extension(),
                Extension::Specified(ext) => builder.with_extension(ext),
            };
            let builder = match value.0.input.first() {
                None => builder,
                Some(x) => builder.with_starting_path(x),
            };
            builder.build()
        }
    }
}
