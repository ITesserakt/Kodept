use clap::Parser;
use rayon::prelude::*;
use tracing::{debug, error};

use cli::common::Kodept;
use kodept_core::loader::Loader;

use crate::cli::commands::{Commands, Execute, Graph};
use crate::stage::PredefinedTraverseSet;

mod cli;
mod stage;

type WideError = anyhow::Error;

fn main() -> Result<(), WideError> {
    let cli_arguments: Kodept = Kodept::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli_arguments.level())
        .init();

    let settings = cli_arguments.diagnostic_config.into();
    let loader: Loader = cli_arguments.loading_config.try_into()?;
    let sources = loader
        .into_sources()
        .into_par_iter()
        .inspect(|source| debug!("Reading {}", source.path()))
        .filter_map(|res| {
            let path = res.path();
            match res.try_into() {
                Ok(source) => Some(source),
                Err(e) => {
                    error!(?path, "Cannot read source, I/O error: {e}.");
                    None
                }
            }
        });

    match cli_arguments.subcommands {
        None => Execute.exec(sources, settings)?,
        Some(Commands::Graph(_)) => Graph::exec(sources, settings, cli_arguments.output)?,
    };
    Ok(())
}
