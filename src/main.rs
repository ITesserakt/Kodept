use clap::Parser;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use tracing::{debug, error};
#[cfg(feature = "profiler")]
use kodept::profiler::HeapProfiler;

use crate::cli::traits::Command;
use cli::common::Kodept;
use kodept::loader::Loader;
use kodept_core::code_source::CodeSource;

mod cli;

type WideError = anyhow::Error;

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
    {    
        HeapProfiler::install();
        ctrlc::set_handler(|| {
        	HeapProfiler::consume();
        	std::process::exit(0);
        })?;
    }

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

    #[cfg(feature = "profiler")]
    {
        HeapProfiler::consume();
    }
    Ok(())
}
