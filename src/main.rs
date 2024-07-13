use clap::Parser;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use tracing::{debug, error};

use cli::common::Kodept;
use kodept::loader::Loader;
use kodept_core::code_source::CodeSource;
use crate::cli::traits::Command;

mod cli;

type WideError = anyhow::Error;

#[cfg(feature = "profiler")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(not(feature = "parallel"))]
fn sources_iter(loader: Loader) -> impl Iterator<Item = CodeSource> {
    loader.into_sources().into_iter()
}

#[cfg(feature = "parallel")]
fn sources_iter(loader: Loader) -> impl ParallelIterator<Item = CodeSource> {
    loader.into_sources().into_par_iter()
}

fn main() -> Result<(), WideError> {
    #[cfg(feature = "profiler")]
    let _profiler = dhat::Profiler::new_heap();

    let cli_arguments: Kodept = Kodept::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli_arguments.level())
        .init();

    let settings = cli_arguments.clone().diagnostic_config.into();
    let loader: Loader = cli_arguments.clone().loading_config.try_into()?;
    let sources = sources_iter(loader)
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

    let args = cli_arguments.clone();
    cli_arguments.subcommands.exec(sources, settings, args)?;
    Ok(())
}
